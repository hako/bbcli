use bbc_news_cli::{app, api, cli, config, events, theme, ui};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use app::App;

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli_args = cli::Cli::parse();

    // If any subcommand is provided, run CLI mode
    if cli_args.command.is_some() || cli_args.feed.is_some() {
        return cli::run_cli(cli_args);
    }

    // Otherwise, launch TUI
    run_tui()
}

fn run_tui() -> Result<()> {
    // Load configuration
    let config = config::load_config().unwrap_or_default();

    // Get theme from config
    let theme = theme::Theme::from_name(&config.theme);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state with theme
    let mut app = App::new(theme);

    // Fetch initial data (both ticker and main feed)
    if let Err(e) = fetch_ticker_data(&mut app) {
        app.set_error(format!("{:#}", e));
    }
    if let Err(e) = fetch_data(&mut app) {
        app.set_error(format!("{:#}", e));
    }

    // Run the app
    let result = run_app(&mut terminal, &mut app, &config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    config: &config::Config,
) -> Result<()> {
    // Initial draw
    terminal.draw(|f| ui::render(f, app))?;

    loop {
        // Track state before event handling to detect changes
        let prev_selected = app.selected;
        let prev_mode = app.mode.clone();
        let prev_preview = app.show_preview;
        let prev_protocol = app.image_protocol.clone();
        let prev_ticker_index = app.ticker_index;
        let prev_loading = app.is_loading;
        let prev_refreshing = app.is_refreshing;
        let prev_humanize_dates = app.humanize_dates;
        let prev_feed_menu_selected = app.feed_menu_selected;
        let prev_show_full_article = app.show_full_article;
        let prev_article_scroll = app.article_scroll_offset;
        let prev_is_fetching_article = app.is_fetching_article;
        let prev_sort_order = app.sort_order.clone();
        let prev_offline = app.is_offline;

        // Check for auto-refresh (every 5 minutes)
        let mut action = events::handle_events(app, config)?;

        if app.check_auto_refresh() && matches!(action, events::AppAction::None) {
            action = events::AppAction::Refresh;
        }

        match action {
            events::AppAction::Refresh => {
                // Keep stories visible during refresh (no loading placeholder)
                app.is_refreshing = true;
                app.clear_error();
                // Redraw to show "Refreshing News..." message
                terminal.draw(|f| ui::render(f, app))?;
                // Refresh both ticker and current feed
                if let Err(e) = fetch_ticker_data(app) {
                    app.set_error(format!("{:#}", e));
                }
                if let Err(e) = fetch_data(app) {
                    app.set_error(format!("{:#}", e));
                }
                app.mark_refreshed();
            }
            events::AppAction::FeedChanged => {
                app.is_loading = true;
                app.is_refreshing = true;
                app.clear_error();
                // Redraw to show "Refreshing News..." message
                terminal.draw(|f| ui::render(f, app))?;
                // Only fetch new feed, keep ticker as-is
                if let Err(e) = fetch_data(app) {
                    app.set_error(format!("{:#}", e));
                }
            }
            events::AppAction::Resize => {
                // No special handling needed, redraw will be triggered below
            }
            events::AppAction::None => {}
        }

        // Update ticker rotation and check if clock should update
        let clock_tick = app.tick();

        // Only redraw if something actually changed
        let should_redraw = prev_selected != app.selected
            || prev_mode != app.mode
            || prev_preview != app.show_preview
            || prev_protocol != app.image_protocol
            || prev_ticker_index != app.ticker_index
            || prev_loading != app.is_loading
            || prev_refreshing != app.is_refreshing
            || prev_humanize_dates != app.humanize_dates
            || prev_feed_menu_selected != app.feed_menu_selected
            || prev_show_full_article != app.show_full_article
            || prev_article_scroll != app.article_scroll_offset
            || prev_is_fetching_article != app.is_fetching_article
            || prev_sort_order != app.sort_order
            || prev_offline != app.is_offline
            || clock_tick  // Redraw when clock ticks (approximately every second)
            || matches!(action, events::AppAction::Refresh | events::AppAction::FeedChanged | events::AppAction::Resize);

        if should_redraw {
            terminal.draw(|f| ui::render(f, app))?;
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn fetch_data(app: &mut App) -> Result<()> {
    // Fetch stories from current feed with cache support
    let feed_url = &app.current_feed.url;

    // First try network fetch directly
    match api::fetch_stories(feed_url) {
        Ok(stories) => {
            // Network is available
            app.is_offline = false;
            app.update_stories(stories);
            Ok(())
        }
        Err(e) => {
            // Network failed, try to use cache
            let cache = bbc_news_cli::cache::Cache::new().ok();
            if let Some(ref cache) = cache {
                if let Some(cached_stories) = cache.load_feed_offline(feed_url) {
                    // We have cached data - we're offline
                    app.is_offline = true;
                    app.update_stories(cached_stories);
                    return Ok(());
                }
            }
            // No cache available either
            Err(e)
        }
    }
}

fn fetch_ticker_data(app: &mut App) -> Result<()> {
    // Always fetch Top Stories for ticker
    const TOP_STORIES_URL: &str = "https://feeds.bbci.co.uk/news/rss.xml";

    // Try network first, fall back to cache if offline
    match api::fetch_stories(TOP_STORIES_URL) {
        Ok(ticker_stories) => {
            app.update_ticker_stories(ticker_stories);
            Ok(())
        }
        Err(e) => {
            // Network failed, try to use cache
            let cache = bbc_news_cli::cache::Cache::new().ok();
            if let Some(ref cache) = cache {
                if let Some(cached_stories) = cache.load_feed_offline(TOP_STORIES_URL) {
                    app.update_ticker_stories(cached_stories);
                    return Ok(());
                }
            }
            // No cache available either
            Err(e)
        }
    }
}
