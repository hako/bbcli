use crate::feeds::{Feed, get_default_feed};
use crate::theme::Theme;
use std::time::{Instant, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    FeedMenu,
    Preview,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Default,         // RSS feed order (as received)
    DateNewest,      // Newest first
    DateOldest,      // Oldest first
}

impl SortOrder {
    pub fn next(&self) -> Self {
        match self {
            SortOrder::Default => SortOrder::DateNewest,
            SortOrder::DateNewest => SortOrder::DateOldest,
            SortOrder::DateOldest => SortOrder::Default,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SortOrder::Default => "Default",
            SortOrder::DateNewest => "Newest First",
            SortOrder::DateOldest => "Oldest First",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageProtocol {
    Auto,       // Automatically detect best protocol
    Halfblocks, // Unicode half blocks (widely compatible)
    Sixel,      // High quality Sixel graphics
    Kitty,      // Kitty graphics protocol (high quality, modern terminals)
}

impl ImageProtocol {
    pub fn next(&self) -> Self {
        match self {
            ImageProtocol::Auto => ImageProtocol::Halfblocks,
            ImageProtocol::Halfblocks => ImageProtocol::Sixel,
            ImageProtocol::Sixel => ImageProtocol::Kitty,
            ImageProtocol::Kitty => ImageProtocol::Auto,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ImageProtocol::Auto => "Auto",
            ImageProtocol::Halfblocks => "Halfblocks",
            ImageProtocol::Sixel => "Sixel",
            ImageProtocol::Kitty => "Kitty",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewsStory {
    pub title: String,
    pub description: String,
    pub link: String,
    pub pub_date: String,
    pub category: String,
    pub image_url: Option<String>,
}

pub struct App {
    pub stories: Vec<NewsStory>,
    pub ticker_stories: Vec<NewsStory>,  // Always contains Top Stories for ticker
    pub selected: usize,
    pub should_quit: bool,
    pub is_loading: bool,
    pub is_refreshing: bool,               // Track if currently refreshing data
    pub is_offline: bool,                  // Track offline mode
    pub error_message: Option<String>,
    pub ticker_index: usize,
    pub ticker_counter: u32,
    pub mode: AppMode,
    pub current_feed: Feed,
    pub feed_menu_selected: usize,
    pub show_preview: bool,
    pub humanize_dates: bool,
    pub image_protocol: ImageProtocol,
    pub theme: Theme,                      // Current theme
    pub show_full_article: bool,           // Toggle between preview and full article view
    pub article_scroll_offset: usize,      // Scroll position in article view
    pub is_fetching_article: bool,         // Loading state for article fetching
    pub sort_order: SortOrder,             // Current sort order
    pub last_refresh_time: Instant,        // Track last refresh for auto-refresh
    article_cache: HashMap<String, String>, // Cache fetched articles by URL
    last_opened_index: Option<usize>,      // Track last opened article index to prevent repeated opens
    last_open_time: Option<Instant>,       // Track last open time for cooldown
    last_selection_change_time: Instant,   // Track when selection last changed
    scroll_count: u32,                     // Count rapid scroll events (for storm detection)
    last_scroll_time: Instant,             // Track last scroll event time
    scroll_pause_until: Option<Instant>,   // When to resume scrolling after storm
}

impl App {
    pub fn new(theme: Theme) -> Self {
        Self {
            stories: Vec::new(),
            ticker_stories: Vec::new(),
            selected: 0,
            should_quit: false,
            is_loading: true,
            is_refreshing: false,                  // Not refreshing initially
            is_offline: false,                     // Start in online mode
            error_message: None,
            ticker_index: 0,
            ticker_counter: 0,
            mode: AppMode::Normal,
            current_feed: get_default_feed(),
            feed_menu_selected: 0,
            show_preview: false,
            humanize_dates: true,  // Default to humanized dates
            image_protocol: ImageProtocol::Auto,  // Auto-detect best protocol
            theme,                                 // Theme from config
            show_full_article: false,              // Start in preview mode
            article_scroll_offset: 0,              // Start at top of article
            is_fetching_article: false,            // Not fetching initially
            sort_order: SortOrder::Default,        // Default RSS order
            last_refresh_time: Instant::now(),     // Initialize to now
            article_cache: HashMap::new(),         // Empty cache
            last_opened_index: None,               // No article opened yet
            last_open_time: None,                  // No article opened yet
            last_selection_change_time: Instant::now(),  // Initialize to now
            scroll_count: 0,                       // No scrolls yet
            last_scroll_time: Instant::now(),      // Initialize to now
            scroll_pause_until: None,              // Not paused
        }
    }

    // Scroll storm detection: prevents catastrophic event buffering in Ghostty
    // Returns true if scrolling should be allowed, false if paused
    fn check_scroll_storm(&mut self) -> bool {
        // Only enable storm detection when preview is OPEN
        // (When preview is closed, scroll keys work normally without blocking)
        if !self.show_preview {
            return true; // Preview closed, allow all scrolling
        }

        // Check if currently paused
        if let Some(pause_until) = self.scroll_pause_until {
            if Instant::now() < pause_until {
                return false; // Still paused, ignore scroll
            } else {
                // Pause expired, resume scrolling
                self.scroll_pause_until = None;
                self.scroll_count = 0;
            }
        }

        // Check if this is part of a scroll storm
        let now = Instant::now();
        let time_since_last_scroll = now.duration_since(self.last_scroll_time).as_secs_f32();

        if time_since_last_scroll > 0.5 {
            // Been a while since last scroll, reset counter
            self.scroll_count = 0;
        }

        // Increment scroll count
        self.scroll_count += 1;
        self.last_scroll_time = now;

        // Check if storm detected (more than 10 scrolls in 0.5 seconds)
        if self.scroll_count > 10 && time_since_last_scroll < 0.5 {
            // SCROLL STORM DETECTED! Pause scrolling for 1 second
            self.scroll_pause_until = Some(now + std::time::Duration::from_secs(1));
            self.scroll_count = 0;
            return false; // Ignore this scroll
        }

        true // Allow scroll
    }

    pub fn next(&mut self) {
        // Check for scroll storm - pause scrolling if detected
        if !self.check_scroll_storm() {
            return; // Scroll storm detected, ignoring this scroll event
        }

        if !self.stories.is_empty() {
            self.selected = (self.selected + 1).min(self.stories.len() - 1);
            // Clear last opened index when selection changes - allows opening new article
            self.last_opened_index = None;
            // Record when selection changed - prevents immediate opens
            self.last_selection_change_time = Instant::now();
        }
    }

    pub fn previous(&mut self) {
        // Check for scroll storm - pause scrolling if detected
        if !self.check_scroll_storm() {
            return; // Scroll storm detected, ignoring this scroll event
        }

        if self.selected > 0 {
            self.selected -= 1;
            // Clear last opened index when selection changes - allows opening new article
            self.last_opened_index = None;
            // Record when selection changed - prevents immediate opens
            self.last_selection_change_time = Instant::now();
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        // Check for scroll storm - pause scrolling if detected
        if !self.check_scroll_storm() {
            return; // Scroll storm detected, ignoring this scroll event
        }

        if !self.stories.is_empty() {
            self.selected = self.stories.len() - 1;
            // Clear last opened index when selection changes - allows opening new article
            self.last_opened_index = None;
            // Record when selection changed - prevents immediate opens
            self.last_selection_change_time = Instant::now();
        }
    }

    pub fn scroll_to_top(&mut self) {
        // Check for scroll storm - pause scrolling if detected
        if !self.check_scroll_storm() {
            return; // Scroll storm detected, ignoring this scroll event
        }

        self.selected = 0;
        // Clear last opened index when selection changes - allows opening new article
        self.last_opened_index = None;
        // Record when selection changed - prevents immediate opens
        self.last_selection_change_time = Instant::now();
    }

    pub fn open_selected(&mut self) -> anyhow::Result<()> {
        // AGGRESSIVE PROTECTION against Ghostty event buffering catastrophe:

        // 1. SELECTION STABILITY: Prevent opening if selection changed recently
        //    (Requires 3 seconds on same article before opening)
        if self.last_selection_change_time.elapsed().as_secs_f32() < 3.0 {
            return Ok(()); // Selection changed too recently, ignore
        }

        // 2. TIME-BASED: Prevent opening ANY article within 5 seconds of last open
        //    (Prevents rapid-fire opens even when scrolling to different articles)
        if let Some(last_time) = self.last_open_time {
            if last_time.elapsed().as_secs_f32() < 5.0 {
                return Ok(()); // Still in cooldown, ignore
            }
        }

        // 3. INDEX-BASED: Prevent opening same article multiple times
        //    (Additional protection against repeated opens)
        if let Some(last_idx) = self.last_opened_index {
            if last_idx == self.selected {
                return Ok(()); // Same article, ignore
            }
        }

        // All checks passed, open the article
        if let Some(story) = self.stories.get(self.selected) {
            webbrowser::open(&story.link)?;
            self.last_opened_index = Some(self.selected);
            self.last_open_time = Some(Instant::now());
        }
        Ok(())
    }

    pub fn open_selected_new_tab(&mut self) -> anyhow::Result<()> {
        // Most browsers will open in a new tab by default
        self.open_selected()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn update_stories(&mut self, stories: Vec<NewsStory>) {
        self.stories = stories;
        self.is_loading = false;
        self.is_refreshing = false;
        // Check selection bounds before sorting
        if self.selected >= self.stories.len() && !self.stories.is_empty() {
            self.selected = self.stories.len() - 1;
        }
        // Re-apply current sort order after updating stories
        // Note: apply_sort() resets selection to 0, which is intentional for sorted views
        if self.sort_order != crate::app::SortOrder::Default {
            self.apply_sort();
        }
    }

    pub fn update_ticker_stories(&mut self, stories: Vec<NewsStory>) {
        self.ticker_stories = stories;
        self.is_refreshing = false;
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
        self.is_loading = false;
        self.is_refreshing = false;
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn tick(&mut self) -> bool {
        self.ticker_counter += 1;
        // Rotate ticker every 100 ticks (approximately 10 seconds at 100ms polling)
        if self.ticker_counter >= 100 {
            self.ticker_counter = 0;
            self.rotate_ticker();
        }
        // Return true every 10 ticks (approximately 1 second) to trigger clock update
        self.ticker_counter % 10 == 0
    }

    fn rotate_ticker(&mut self) {
        if self.ticker_stories.is_empty() {
            self.ticker_index = 0;
            return;
        }

        let max_ticker_items = 5.min(self.ticker_stories.len());
        self.ticker_index = (self.ticker_index + 1) % max_ticker_items;
    }

    pub fn toggle_feed_menu(&mut self) {
        self.mode = if self.mode == AppMode::FeedMenu {
            AppMode::Normal
        } else {
            AppMode::FeedMenu
        };
    }

    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    pub fn toggle_date_format(&mut self) {
        self.humanize_dates = !self.humanize_dates;
    }

    pub fn cycle_image_protocol(&mut self) {
        self.image_protocol = self.image_protocol.next();
    }

    pub fn feed_menu_next(&mut self, feed_count: usize) {
        if feed_count > 0 {
            self.feed_menu_selected = (self.feed_menu_selected + 1).min(feed_count - 1);
        }
    }

    pub fn feed_menu_previous(&mut self) {
        if self.feed_menu_selected > 0 {
            self.feed_menu_selected -= 1;
        }
    }

    pub fn select_feed(&mut self, feed: Feed) {
        self.current_feed = feed;
        self.mode = AppMode::Normal;
        self.is_loading = true;
        self.selected = 0;
    }

    // Article viewing methods
    pub fn fetch_and_show_article(&mut self) {
        if let Some(story) = self.stories.get(self.selected) {
            let url = &story.link;

            // Check cache first
            if !self.article_cache.contains_key(url) {
                // Not in cache, fetch it
                self.is_fetching_article = true;

                use crate::article_fetcher::fetch_article_content;
                match fetch_article_content(url) {
                    Ok(content) => {
                        self.article_cache.insert(url.clone(), content);
                        self.is_fetching_article = false;
                        self.show_full_article = true;
                        self.article_scroll_offset = 0;
                    }
                    Err(e) => {
                        self.is_fetching_article = false;
                        self.set_error(format!("Failed to fetch article: {}", e));
                    }
                }
            } else {
                // Already cached, show it
                self.show_full_article = true;
                self.article_scroll_offset = 0;
            }
        }
    }

    pub fn toggle_article_view(&mut self) {
        if self.show_full_article {
            // Return to preview
            self.show_full_article = false;
            self.article_scroll_offset = 0;
        }
    }

    pub fn scroll_article_up(&mut self) {
        if self.article_scroll_offset > 0 {
            self.article_scroll_offset = self.article_scroll_offset.saturating_sub(1);
        }
    }

    pub fn scroll_article_down(&mut self) {
        // Scroll down (limit will be checked during rendering)
        self.article_scroll_offset += 1;
    }

    pub fn get_current_article_text(&self) -> Option<&String> {
        if let Some(story) = self.stories.get(self.selected) {
            self.article_cache.get(&story.link)
        } else {
            None
        }
    }

    pub fn toggle_help_menu(&mut self) {
        self.mode = if self.mode == AppMode::Help {
            AppMode::Normal
        } else {
            AppMode::Help
        };
    }

    pub fn cycle_theme(&mut self) {
        self.theme = match self.theme.name.as_str() {
            "Light" => Theme::dark(),
            "Dark" => Theme::light(),
            _ => Theme::light(),
        };
    }

    pub fn cycle_sort_order(&mut self) {
        self.sort_order = self.sort_order.next();
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        match self.sort_order {
            SortOrder::Default => {
                // Keep original RSS order - do nothing or reload
            }
            SortOrder::DateNewest => {
                // Sort by date, newest first
                self.stories.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
            }
            SortOrder::DateOldest => {
                // Sort by date, oldest first
                self.stories.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
            }
        }
        // Reset selection to top after sorting
        self.selected = 0;
    }

    pub fn check_auto_refresh(&self) -> bool {
        // Auto-refresh every 5 minutes (300 seconds)
        const AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(300);
        self.last_refresh_time.elapsed() >= AUTO_REFRESH_INTERVAL
    }

    pub fn mark_refreshed(&mut self) {
        self.last_refresh_time = Instant::now();
    }

    // Jump to the current ticker article
    // Returns true if feed needs to change (trigger FeedChanged action)
    pub fn jump_to_ticker_article(&mut self) -> bool {
        // Check if ticker has any stories
        if self.ticker_stories.is_empty() || self.ticker_index >= self.ticker_stories.len() {
            return false;
        }

        // Get the current ticker story
        let ticker_story = &self.ticker_stories[self.ticker_index];
        let ticker_url = &ticker_story.link;

        // Check if we're already on Top Stories feed
        let top_stories_feed = crate::feeds::get_default_feed();
        let need_feed_change = self.current_feed.url != top_stories_feed.url;

        if need_feed_change {
            // Switch to Top Stories feed
            self.current_feed = top_stories_feed;
            self.is_loading = true;
            self.selected = 0;
            return true; // Signal that feed needs to be fetched
        } else {
            // Already on Top Stories, just find and select the ticker article
            if let Some(index) = self.stories.iter().position(|s| s.link == *ticker_url) {
                self.selected = index;
                // Clear last opened to allow opening this article
                self.last_opened_index = None;
                self.last_selection_change_time = Instant::now();
            } else {
                // Story not found, select first story
                self.selected = 0;
            }
            return false;
        }
    }
}
