use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::NewsStory;

const CACHE_EXPIRY_SECS: u64 = 900; // 15 minutes

#[derive(Serialize, Deserialize)]
struct CachedFeed {
    stories: Vec<NewsStory>,
    timestamp: u64,
    feed_url: String,
}

#[derive(Serialize, Deserialize)]
struct CachedArticle {
    content: String,
    timestamp: u64,
    url: String,
}

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new() -> Result<Self> {
        // Use ~/.bbcli/cache for consistency with config location
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let cache_dir = home.join(".bbcli").join("cache");

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)?;

        Ok(Cache { cache_dir })
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get cache file path for a feed
    fn feed_cache_path(&self, feed_url: &str) -> PathBuf {
        let hash = Self::hash_string(feed_url);
        self.cache_dir.join(format!("feed_{}.bin", hash))
    }

    /// Get cache file path for an article
    fn article_cache_path(&self, article_url: &str) -> PathBuf {
        let hash = Self::hash_string(article_url);
        self.cache_dir.join(format!("article_{}.bin", hash))
    }

    /// Simple hash function for URLs
    fn hash_string(s: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Save feed to cache
    pub fn save_feed(&self, feed_url: &str, stories: &[NewsStory]) -> Result<()> {
        let cached_feed = CachedFeed {
            stories: stories.to_vec(),
            timestamp: Self::current_timestamp(),
            feed_url: feed_url.to_string(),
        };

        let path = self.feed_cache_path(feed_url);
        let encoded = bincode::serialize(&cached_feed)?;
        fs::write(path, encoded)?;

        Ok(())
    }

    /// Load feed from cache if not expired
    pub fn load_feed(&self, feed_url: &str) -> Option<Vec<NewsStory>> {
        let path = self.feed_cache_path(feed_url);

        if !path.exists() {
            return None;
        }

        let data = fs::read(&path).ok()?;
        let cached_feed: CachedFeed = bincode::deserialize(&data).ok()?;

        // Check if cache is expired
        let age = Self::current_timestamp() - cached_feed.timestamp;
        if age > CACHE_EXPIRY_SECS {
            return None;
        }

        Some(cached_feed.stories)
    }

    /// Load feed from cache regardless of expiry (for offline mode)
    pub fn load_feed_offline(&self, feed_url: &str) -> Option<Vec<NewsStory>> {
        let path = self.feed_cache_path(feed_url);

        if !path.exists() {
            return None;
        }

        let data = fs::read(&path).ok()?;
        let cached_feed: CachedFeed = bincode::deserialize(&data).ok()?;

        Some(cached_feed.stories)
    }

    /// Save article content to cache
    pub fn save_article(&self, article_url: &str, content: &str) -> Result<()> {
        let cached_article = CachedArticle {
            content: content.to_string(),
            timestamp: Self::current_timestamp(),
            url: article_url.to_string(),
        };

        let path = self.article_cache_path(article_url);
        let encoded = bincode::serialize(&cached_article)?;
        fs::write(path, encoded)?;

        Ok(())
    }

    /// Load article from cache if not expired
    pub fn load_article(&self, article_url: &str) -> Option<String> {
        let path = self.article_cache_path(article_url);

        if !path.exists() {
            return None;
        }

        let data = fs::read(&path).ok()?;
        let cached_article: CachedArticle = bincode::deserialize(&data).ok()?;

        // Check if cache is expired (articles cache for longer - 1 hour)
        let age = Self::current_timestamp() - cached_article.timestamp;
        if age > 3600 {
            return None;
        }

        Some(cached_article.content)
    }

    /// Load article from cache regardless of expiry (for offline mode)
    pub fn load_article_offline(&self, article_url: &str) -> Option<String> {
        let path = self.article_cache_path(article_url);

        if !path.exists() {
            return None;
        }

        let data = fs::read(&path).ok()?;
        let cached_article: CachedArticle = bincode::deserialize(&data).ok()?;

        Some(cached_article.content)
    }

    /// Get cache age in seconds for a feed
    pub fn get_feed_age(&self, feed_url: &str) -> Option<u64> {
        let path = self.feed_cache_path(feed_url);

        if !path.exists() {
            return None;
        }

        let data = fs::read(&path).ok()?;
        let cached_feed: CachedFeed = bincode::deserialize(&data).ok()?;

        Some(Self::current_timestamp() - cached_feed.timestamp)
    }

    /// Clear all cache
    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
}
