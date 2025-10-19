#[derive(Debug, Clone)]
pub struct Feed {
    pub name: String,
    pub url: String,
}

impl Feed {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

pub fn get_all_feeds() -> Vec<Feed> {
    vec![
        Feed::new("Top Stories", "https://feeds.bbci.co.uk/news/rss.xml"),
        Feed::new("World", "https://feeds.bbci.co.uk/news/world/rss.xml"),
        Feed::new("UK", "https://feeds.bbci.co.uk/news/uk/rss.xml"),
        Feed::new("Business", "https://feeds.bbci.co.uk/news/business/rss.xml"),
        Feed::new("Politics", "https://feeds.bbci.co.uk/news/politics/rss.xml"),
        Feed::new("Health", "https://feeds.bbci.co.uk/news/health/rss.xml"),
        Feed::new("Education & Family", "https://feeds.bbci.co.uk/news/education/rss.xml"),
        Feed::new("Science & Environment", "https://feeds.bbci.co.uk/news/science_and_environment/rss.xml"),
        Feed::new("Technology", "https://feeds.bbci.co.uk/news/technology/rss.xml"),
        Feed::new("Entertainment & Arts", "https://feeds.bbci.co.uk/news/entertainment_and_arts/rss.xml"),
        Feed::new("England", "https://feeds.bbci.co.uk/news/england/rss.xml"),
        Feed::new("Northern Ireland", "https://feeds.bbci.co.uk/news/northern_ireland/rss.xml"),
        Feed::new("Scotland", "https://feeds.bbci.co.uk/news/scotland/rss.xml"),
        Feed::new("Wales", "https://feeds.bbci.co.uk/news/wales/rss.xml"),
        Feed::new("Africa", "https://feeds.bbci.co.uk/news/world/africa/rss.xml"),
        Feed::new("Asia", "https://feeds.bbci.co.uk/news/world/asia/rss.xml"),
        Feed::new("Europe", "https://feeds.bbci.co.uk/news/world/europe/rss.xml"),
        Feed::new("Latin America", "https://feeds.bbci.co.uk/news/world/latin_america/rss.xml"),
        Feed::new("Middle East", "https://feeds.bbci.co.uk/news/world/middle_east/rss.xml"),
        Feed::new("US & Canada", "https://feeds.bbci.co.uk/news/world/us_and_canada/rss.xml"),
    ]
}

pub fn get_default_feed() -> Feed {
    Feed::new("Top Stories", "https://feeds.bbci.co.uk/news/rss.xml")
}

pub fn get_feed_by_name(name: &str) -> anyhow::Result<Feed> {
    let name_lower = name.to_lowercase();
    let feeds = get_all_feeds();

    // Try exact match first (case-insensitive)
    for feed in &feeds {
        if feed.name.to_lowercase() == name_lower {
            return Ok(feed.clone());
        }
    }

    // Try partial match (shortcuts)
    for feed in &feeds {
        if feed.name.to_lowercase().contains(&name_lower) {
            return Ok(feed.clone());
        }
    }

    // Try common shortcuts
    let feed = match name_lower.as_str() {
        "tech" => Feed::new("Technology", "https://feeds.bbci.co.uk/news/technology/rss.xml"),
        "biz" | "business" => Feed::new("Business", "https://feeds.bbci.co.uk/news/business/rss.xml"),
        "pol" | "politics" => Feed::new("Politics", "https://feeds.bbci.co.uk/news/politics/rss.xml"),
        "sci" | "science" => Feed::new("Science & Environment", "https://feeds.bbci.co.uk/news/science_and_environment/rss.xml"),
        "entertainment" | "ent" => Feed::new("Entertainment & Arts", "https://feeds.bbci.co.uk/news/entertainment_and_arts/rss.xml"),
        "edu" | "education" => Feed::new("Education & Family", "https://feeds.bbci.co.uk/news/education/rss.xml"),
        _ => anyhow::bail!("Unknown feed: '{}'. Use 'world', 'uk', 'business', 'technology', etc.", name),
    };

    Ok(feed)
}
