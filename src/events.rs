use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;

use crate::app::{App, AppMode};
use crate::config::Config;
use crate::feeds::get_all_feeds;

pub enum AppAction {
    None,
    Refresh,
    FeedChanged,
    Resize,
}

pub fn handle_events(app: &mut App, config: &Config) -> anyhow::Result<AppAction> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                return handle_key_event(app, key, config);
            }
            Event::Resize(_, _) => {
                return Ok(AppAction::Resize);
            }
            _ => {}
        }
    }
    Ok(AppAction::None)
}

fn handle_key_event(app: &mut App, key: KeyEvent, config: &Config) -> anyhow::Result<AppAction> {
    let kb = &config.keybindings;

    // Handle help menu mode separately
    if app.mode == AppMode::Help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => app.toggle_help_menu(),
            _ => {}
        }
        return Ok(AppAction::None);
    }

    // Handle feed menu mode separately
    if app.mode == AppMode::FeedMenu {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let feeds = get_all_feeds();
                app.feed_menu_next(feeds.len());
            }
            KeyCode::Char('k') | KeyCode::Up => app.feed_menu_previous(),
            KeyCode::Enter => {
                let feeds = get_all_feeds();
                if let Some(feed) = feeds.get(app.feed_menu_selected) {
                    app.select_feed(feed.clone());
                    return Ok(AppAction::FeedChanged);
                }
            }
            KeyCode::Char('f') | KeyCode::Esc => app.toggle_feed_menu(),
            _ => {}
        }
        return Ok(AppAction::None);
    }

    // ARTICLE VIEW MODE: Handle scrolling within article
    if app.show_full_article {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => app.scroll_article_up(),
            KeyCode::Down | KeyCode::Char('j') => app.scroll_article_down(),
            KeyCode::Char(c) if c == kb.open => app.open_selected()?,
            KeyCode::Enter | KeyCode::Tab | KeyCode::Esc => app.toggle_article_view(),
            _ => {}
        }
        return Ok(AppAction::None);
    }

    // PREVIEW PANE PROTECTION: Block scrolling keys when preview is open
    // This prevents Ghostty event buffering catastrophe (scroll storm detection is backup)
    if app.show_preview && !app.show_full_article {
        let is_scroll_key = matches!(
            key.code,
            KeyCode::Down | KeyCode::Up
        ) || matches!(
            key.code,
            KeyCode::Char(c) if c == kb.scroll_down || c == kb.scroll_up || c == kb.scroll_bottom || c == kb.latest
        );

        if is_scroll_key {
            // Drain all buffered scroll events to prevent delay when Tab is pressed
            // This prevents 5+ second delays when keys were held down
            while event::poll(Duration::ZERO)? {
                if let Event::Key(next_key) = event::read()? {
                    if next_key.kind == KeyEventKind::Press {
                        // Check if this is also a scroll key
                        let is_next_scroll = matches!(
                            next_key.code,
                            KeyCode::Down | KeyCode::Up
                        ) || matches!(
                            next_key.code,
                            KeyCode::Char(c) if c == kb.scroll_down || c == kb.scroll_up || c == kb.scroll_bottom || c == kb.latest
                        );

                        if !is_next_scroll {
                            // Found a non-scroll key (like Tab!), process it immediately
                            return handle_key_event(app, next_key, config);
                        }
                        // Otherwise it's another scroll key, discard and continue draining
                    }
                }
            }
            // All buffered scroll events drained, return
            return Ok(AppAction::None);
        }
    }

    // Normal mode key handling
    match key.code {
        KeyCode::Char(c) if c == kb.quit => app.quit(),
        KeyCode::Char(c) if c == kb.scroll_down => app.next(),
        KeyCode::Char(c) if c == kb.scroll_up => app.previous(),
        KeyCode::Char(c) if c == kb.scroll_bottom => app.scroll_to_bottom(),
        KeyCode::Char(c) if c == kb.open => app.open_selected()?,
        KeyCode::Char(c) if c == kb.open_new_tab => app.open_selected_new_tab()?,
        KeyCode::Char(c) if c == kb.latest => app.scroll_to_top(),
        KeyCode::Char(c) if c == kb.refresh => return Ok(AppAction::Refresh),
        KeyCode::Char('f') => app.toggle_feed_menu(),
        KeyCode::Char('s') => app.cycle_sort_order(),
        KeyCode::Char('t') => app.toggle_date_format(),
        KeyCode::Char('T') => app.cycle_theme(),
        KeyCode::Char('p') => app.cycle_image_protocol(),
        KeyCode::Char('?') => app.toggle_help_menu(),
        KeyCode::Char('a') => app.fetch_and_show_article(),
        KeyCode::Char(' ') => {
            // Jump to current ticker article
            if app.jump_to_ticker_article() {
                return Ok(AppAction::FeedChanged);
            }
        }
        KeyCode::Tab => app.toggle_preview(),
        KeyCode::Enter => app.fetch_and_show_article(),
        // Also support arrow keys
        KeyCode::Down => app.next(),
        KeyCode::Up => app.previous(),
        KeyCode::Esc => app.quit(),
        _ => {}
    }

    Ok(AppAction::None)
}
