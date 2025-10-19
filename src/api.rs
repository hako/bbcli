use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::time::Duration;

use crate::app::NewsStory;
use crate::cache::Cache;

fn create_http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 10; SM-A307G) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36")
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to create HTTP client")
}

pub fn fetch_stories(feed_url: &str) -> Result<Vec<NewsStory>> {
    fetch_stories_with_cache(feed_url, false)
}

pub fn fetch_stories_with_cache(feed_url: &str, force_offline: bool) -> Result<Vec<NewsStory>> {
    let cache = Cache::new().ok();

    // Try to load from cache first (if not expired)
    if let Some(ref cache) = cache {
        if let Some(cached_stories) = cache.load_feed(feed_url) {
            return Ok(cached_stories);
        }
    }

    // Try to fetch from network
    let stories_result = if !force_offline {
        fetch_from_network(feed_url)
    } else {
        Err(anyhow::anyhow!("Offline mode - skipping network fetch"))
    };

    match stories_result {
        Ok(stories) => {
            // Save to cache on successful fetch
            if let Some(ref cache) = cache {
                let _ = cache.save_feed(feed_url, &stories);
            }
            Ok(stories)
        }
        Err(e) => {
            // Network failed, try to load from cache (even if expired)
            if let Some(ref cache) = cache {
                if let Some(cached_stories) = cache.load_feed_offline(feed_url) {
                    return Ok(cached_stories);
                }
            }
            Err(e)
        }
    }
}

fn fetch_from_network(feed_url: &str) -> Result<Vec<NewsStory>> {
    let client = create_http_client()?;

    let response = client
        .get(feed_url)
        .send()
        .context("Failed to fetch BBC RSS feed")?
        .text()
        .context("Failed to read response text")?;

    parse_rss(&response)
}

fn parse_rss(xml_content: &str) -> Result<Vec<NewsStory>> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut stories = Vec::new();
    let mut buf = Vec::new();

    let mut in_item = false;
    let mut current_story = NewsStory {
        title: String::new(),
        description: String::new(),
        link: String::new(),
        pub_date: String::new(),
        category: String::new(),
        image_url: None,
    };

    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    in_item = true;
                    current_story = NewsStory {
                        title: String::new(),
                        description: String::new(),
                        link: String::new(),
                        pub_date: String::new(),
                        category: String::from("News"),
                        image_url: None,
                    };
                } else if in_item {
                    // Handle media:thumbnail tag to extract image URL
                    if tag_name == "media:thumbnail" || tag_name == "media:content" {
                        // Extract url attribute
                        if let Some(url_attr) = e.attributes()
                            .filter_map(|a| a.ok())
                            .find(|attr| {
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                key == "url"
                            })
                        {
                            if let Ok(url_value) = url_attr.unescape_value() {
                                current_story.image_url = Some(url_value.to_string());
                            }
                        }
                    }
                    current_tag = tag_name;
                }
            }
            Ok(Event::Text(e)) => {
                if in_item {
                    let text = e.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "title" => current_story.title = text,
                        "description" => current_story.description = text,
                        "link" => current_story.link = text,
                        "pubDate" => current_story.pub_date = format_date(&text),
                        "category" => current_story.category = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::CData(e)) => {
                if in_item {
                    let text = String::from_utf8_lossy(&e.into_inner()).to_string();
                    match current_tag.as_str() {
                        "title" => current_story.title = text,
                        "description" => current_story.description = text,
                        "link" => current_story.link = text,
                        "pubDate" => current_story.pub_date = format_date(&text),
                        "category" => current_story.category = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                // Handle self-closing tags like <media:thumbnail ... />
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if in_item && (tag_name == "media:thumbnail" || tag_name == "media:content") {
                    // Extract url attribute
                    if let Some(url_attr) = e.attributes()
                        .filter_map(|a| a.ok())
                        .find(|attr| {
                            let key = String::from_utf8_lossy(attr.key.as_ref());
                            key == "url"
                        })
                    {
                        if let Ok(url_value) = url_attr.unescape_value() {
                            current_story.image_url = Some(url_value.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    in_item = false;
                    if !current_story.title.is_empty() {
                        stories.push(current_story.clone());
                    }
                } else if in_item {
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("Error parsing XML at position {}: {:?}", reader.buffer_position(), e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(stories.into_iter().take(30).collect())
}

fn format_date(date_str: &str) -> String {
    use chrono::DateTime;

    // BBC RSS dates are in RFC 2822 format (e.g., "Wed, 03 Feb 2015 15:58:15 GMT")
    // Convert to "YYYY-MM-DD HH:MM:SS" format
    if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        date_str.to_string()
    }
}
