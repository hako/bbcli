use anyhow::Result;
use reqwest::blocking::Client;
use dom_smoothie::Readability;
use html2text::from_read;
use std::time::Duration;

use crate::cache::Cache;

pub fn fetch_article_content(url: &str) -> Result<String> {
    let cache = Cache::new().ok();

    // Try to load from cache first
    if let Some(ref cache) = cache {
        if let Some(cached_content) = cache.load_article(url) {
            return Ok(cached_content);
        }
    }

    // Try to fetch from network
    let content_result = fetch_article_from_network(url);

    match content_result {
        Ok(content) => {
            // Save to cache on successful fetch
            if let Some(ref cache) = cache {
                let _ = cache.save_article(url, &content);
            }
            Ok(content)
        }
        Err(e) => {
            // Network failed, try to load from cache (even if expired)
            if let Some(ref cache) = cache {
                if let Some(cached_content) = cache.load_article_offline(url) {
                    return Ok(cached_content);
                }
            }
            Err(e)
        }
    }
}

fn fetch_article_from_network(url: &str) -> Result<String> {
    // Fetch HTML content
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send()?;
    let html = response.text()?;

    // Extract article using dom_smoothie
    let mut readability = Readability::new(html, None, None)?;
    let article = readability.parse()?;

    // Get the cleaned HTML content as a string
    let cleaned_html = article.content.to_string();

    // Convert cleaned HTML to plain text
    // Use large width to avoid hard-wrapping - let the UI handle text wrapping
    let text = from_read(cleaned_html.as_bytes(), 1000);

    // Create formatted article with title
    let title = article.title;
    let formatted = format!(
        "{}\n{}\n\n{}",
        title,
        "=".repeat(title.len()),
        text.trim()
    );

    Ok(formatted)
}
