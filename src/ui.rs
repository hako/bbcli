use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Clear},
    Frame,
};
use ratatui_image::{picker::Picker, StatefulImage};

use crate::app::{App, AppMode, ImageProtocol};
use crate::date_utils::humanize_time;
use crate::feeds::get_all_feeds;
use crate::image_cache::get_image;
use ratatui_image::picker::ProtocolType;

pub fn render(f: &mut Frame, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Footer (no border)
        ])
        .split(f.area());

    render_header(f, main_chunks[0], app);

    // Full article view takes over the entire content area (full width)
    if app.show_full_article {
        render_full_article(f, main_chunks[1], app);
    } else if app.show_preview {
        // Split main content area if preview is enabled (but not full article)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80),  // Story list
                Constraint::Percentage(20),  // Preview
            ])
            .split(main_chunks[1]);

        render_stories(f, content_chunks[0], app);
        render_preview(f, content_chunks[1], app);
    } else {
        render_stories(f, main_chunks[1], app);
    }

    render_footer(f, main_chunks[2], app);

    // Render feed menu overlay if in feed menu mode
    if app.mode == AppMode::FeedMenu {
        render_feed_menu(f, app);
    }

    // Render help menu overlay if in help mode
    if app.mode == AppMode::Help {
        render_help_menu(f, app);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let last_updated = if !app.stories.is_empty() {
        let date_str = app.stories.first().map(|s| s.pub_date.as_str()).unwrap_or("");
        let formatted_date = if app.humanize_dates {
            humanize_time(date_str)
        } else {
            date_str.to_string()
        };
        format!("Last updated: {} | {}", formatted_date, app.current_feed.name)
    } else {
        format!("Last updated: -- | {}", app.current_feed.name)
    };

    // Add offline indicator if in offline mode
    let title_text = if app.is_offline {
        "BBC | NEWS [OFFLINE]"
    } else {
        "BBC | NEWS"
    };

    let header_text = vec![
        Line::from(
            Span::styled(
                title_text,
                Style::default().fg(app.theme.accent_fg).add_modifier(Modifier::BOLD)
            )
        ),
        Line::from(
            Span::styled(
                last_updated,
                Style::default().fg(app.theme.accent_fg)
            )
        ),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(app.theme.accent)));

    f.render_widget(header, area);
}

fn render_stories(f: &mut Frame, area: Rect, app: &App) {
    if app.is_loading {
        let loading = Paragraph::new("Loading BBC News...")
            .style(Style::default().fg(app.theme.fg_primary).bg(app.theme.bg_primary))
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
        return;
    }

    // Show error message if present
    if let Some(ref error_msg) = app.error_message {
        let error_text = vec![
            Line::from(Span::styled("Error fetching BBC News:", Style::default().fg(Color::Red).bg(app.theme.bg_primary).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(error_msg, Style::default().fg(app.theme.fg_primary).bg(app.theme.bg_primary))),
            Line::from(""),
            Line::from(Span::styled("Press 'r' to retry", Style::default().fg(app.theme.fg_secondary).bg(app.theme.bg_primary))),
        ];
        let error = Paragraph::new(error_text)
            .style(Style::default().bg(app.theme.bg_primary))
            .alignment(Alignment::Center);
        f.render_widget(error, area);
        return;
    }

    if app.stories.is_empty() {
        let empty = Paragraph::new("No stories available. Press 'r' to refresh.")
            .style(Style::default().fg(Color::Red).bg(app.theme.bg_primary))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .stories
        .iter()
        .enumerate()
        .map(|(i, story)| {
            let is_selected = i == app.selected;
            let number = i + 1;

            let title_text = format!("{}. {}", number, story.title);

            // Pad title to full width for full-width background, truncate if too long
            let width = area.width as usize;
            let padded_title = if title_text.len() < width {
                format!("{}{}", title_text, " ".repeat(width - title_text.len()))
            } else {
                // Truncate and add ellipsis
                let max_len = width.saturating_sub(3);
                if max_len > 0 {
                    let truncated = title_text.chars().take(max_len).collect::<String>();
                    format!("{}...", truncated)
                } else {
                    title_text.chars().take(width).collect::<String>()
                }
            };

            let title_line = if is_selected {
                // Selected: white text on accent (BBC red) background (full width)
                Line::styled(
                    padded_title,
                    Style::default()
                        .fg(app.theme.accent_fg)
                        .bg(app.theme.accent)
                        .add_modifier(Modifier::BOLD)
                )
            } else {
                // Normal: primary text on primary background
                Line::styled(
                    padded_title,
                    Style::default()
                        .fg(app.theme.fg_primary)
                        .bg(app.theme.bg_primary)
                        .add_modifier(Modifier::BOLD)
                )
            };

            // Metadata line (indented with 3 spaces) - always gray background
            let formatted_date = if app.humanize_dates {
                humanize_time(&story.pub_date)
            } else {
                story.pub_date.clone()
            };
            let meta_text = format!("   Last updated: {} | {}", formatted_date, app.current_feed.name);
            let padded_meta = if meta_text.len() < width {
                format!("{}{}", meta_text, " ".repeat(width - meta_text.len()))
            } else {
                // Truncate and add ellipsis
                let max_len = width.saturating_sub(3);
                if max_len > 0 {
                    let truncated = meta_text.chars().take(max_len).collect::<String>();
                    format!("{}...", truncated)
                } else {
                    meta_text.chars().take(width).collect::<String>()
                }
            };

            let meta_line = Line::styled(
                padded_meta,
                Style::default()
                    .fg(app.theme.fg_secondary)
                    .bg(app.theme.bg_primary)
            );

            ListItem::new(vec![title_line, meta_line])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(app.theme.bg_primary)));

    // Create state for scrolling
    let mut state = ListState::default();
    state.select(Some(app.selected));

    f.render_stateful_widget(list, area, &mut state);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    use chrono::Local;

    // Split footer into left and right sections
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),      // Left: status/ticker
            Constraint::Length(8),   // Right: time (HH:MM:SS)
        ])
        .split(area);

    // Left side: show refresh status or ticker/keybindings
    let footer_text = if app.is_refreshing {
        String::from("Refreshing News...")
    } else if !app.ticker_stories.is_empty() {
        let max_ticker_items = 8.min(app.ticker_stories.len());
        if app.ticker_index < app.ticker_stories.len() {
            let story = &app.ticker_stories[app.ticker_index];
            format!("[LATEST] {} ({}/{})", story.title, app.ticker_index + 1, max_ticker_items)
        } else {
            format!("q: quit | o: open | Tab: preview | f: feeds | s: sort ({}) | t: dates | p: protocol ({}) | r: refresh", app.sort_order.name(), app.image_protocol.name())
        }
    } else {
        // Show default keybindings help when no ticker stories
        format!("q: quit | o: open | Tab: preview | f: feeds | s: sort ({}) | t: dates | p: protocol ({}) | r: refresh", app.sort_order.name(), app.image_protocol.name())
    };

    let footer_left = Paragraph::new(footer_text)
        .style(Style::default().fg(app.theme.fg_primary).bg(app.theme.bg_accent))
        .alignment(Alignment::Left);

    // Right side: current time with seconds
    let current_time = Local::now().format("%H:%M:%S").to_string();
    let footer_right = Paragraph::new(current_time)
        .style(Style::default().fg(app.theme.fg_primary).bg(app.theme.bg_accent))
        .alignment(Alignment::Right);

    f.render_widget(footer_left, footer_chunks[0]);
    f.render_widget(footer_right, footer_chunks[1]);
}

fn render_full_article(f: &mut Frame, area: Rect, app: &App) {
    if let Some(_story) = app.stories.get(app.selected) {
        let article_block = Block::default()
            .title("Article View (Enter/Tab/Esc to close)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.accent))
            .style(Style::default().bg(app.theme.bg_primary));

        // Get inner area (within borders)
        let inner_area = article_block.inner(area);
        f.render_widget(article_block, area);

        if let Some(article_text) = app.get_current_article_text() {
            // Create paragraph with full text and let ratatui handle wrapping
            let article_paragraph = Paragraph::new(article_text.as_str())
                .wrap(ratatui::widgets::Wrap { trim: true })
                .alignment(Alignment::Left)
                .style(Style::default().fg(app.theme.fg_primary).bg(app.theme.bg_primary))
                .scroll((app.article_scroll_offset as u16, 0));

            f.render_widget(article_paragraph, inner_area);
        } else {
            // No article content cached - show error message
            let error_msg = Paragraph::new("No article content available.\nPress Tab or Esc to return to preview.")
                .style(Style::default().fg(Color::Red).bg(app.theme.bg_primary))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(error_msg, inner_area);
        }
    }
}

fn render_preview(f: &mut Frame, area: Rect, app: &App) {
    if let Some(story) = app.stories.get(app.selected) {
        // Choose title based on loading state
        let title = if app.is_fetching_article {
            "Loading Article..."
        } else {
            "Preview (Tab/Enter for article)"
        };

        let preview_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.accent))
            .style(Style::default().bg(app.theme.bg_primary));

        // Get inner area (within borders)
        let inner_area = preview_block.inner(area);
        f.render_widget(preview_block, area);

        // LOADING STATE: Show loading message
        if app.is_fetching_article {
            let loading_msg = Paragraph::new("Fetching article...\nThis may take a few seconds.")
                .style(Style::default().fg(app.theme.fg_secondary).bg(app.theme.bg_primary))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(loading_msg, inner_area);
            return;
        }

        // Check if area is too small for preview
        let min_width = 20;
        let min_height = 6;

        if area.width < min_width || area.height < min_height {
            // Too small - show message instead
            let msg = Paragraph::new("Terminal too small for preview")
                .style(Style::default().fg(app.theme.fg_secondary).bg(app.theme.bg_primary))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(msg, inner_area);
            return;
        }

        // Split into image area and text area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Image area
                Constraint::Percentage(60), // Text area
            ])
            .split(inner_area);

        // Render image
        let img = get_image(story.image_url.as_deref());

        // Get image area for rendering
        let image_area = chunks[0];

        // Create picker and configure protocol
        let mut picker = Picker::from_termios().unwrap_or_else(|_| Picker::new((8, 12)));

        // Set protocol based on user preference
        match app.image_protocol {
            ImageProtocol::Auto => {
                picker.guess_protocol();
            },
            ImageProtocol::Halfblocks => {
                picker.protocol_type = ProtocolType::Halfblocks;
            },
            ImageProtocol::Sixel => {
                picker.protocol_type = ProtocolType::Sixel;
            },
            ImageProtocol::Kitty => {
                picker.protocol_type = ProtocolType::Kitty;
            },
        }

        // Calculate target pixel dimensions based on widget area and font size
        // This ensures the image fits within the allocated space while maintaining aspect ratio
        let font_size = picker.font_size;
        let target_width = (image_area.width as u32 * font_size.0 as u32) as u32;
        let target_height = (image_area.height as u32 * font_size.1 as u32) as u32;

        // Resize image to fit within the widget area while maintaining aspect ratio
        let resized_img = img.resize(target_width, target_height, image::imageops::FilterType::Triangle);

        // Create protocol from resized image
        let mut dyn_img = picker.new_resize_protocol(resized_img);
        let image_widget = StatefulImage::new(None);
        f.render_stateful_widget(image_widget, image_area, &mut dyn_img);

        // Create text content
        let mut preview_lines = vec![
            Line::from(Span::styled(
                &story.title,
                Style::default()
                    .fg(app.theme.fg_primary)
                    .add_modifier(Modifier::BOLD)
            )),
            Line::from(""),
        ];

        // Add description
        if !story.description.is_empty() {
            preview_lines.push(Line::from(Span::styled(
                &story.description,
                Style::default().fg(app.theme.fg_primary)
            )));
            preview_lines.push(Line::from(""));
        }

        // Add metadata
        let formatted_date = if app.humanize_dates {
            humanize_time(&story.pub_date)
        } else {
            story.pub_date.clone()
        };
        preview_lines.push(Line::from(Span::styled(
            format!("Published: {}", formatted_date),
            Style::default().fg(app.theme.fg_secondary)
        )));
        preview_lines.push(Line::from(Span::styled(
            format!("Feed: {}", app.current_feed.name),
            Style::default().fg(app.theme.fg_secondary)
        )));

        let preview_text = Paragraph::new(preview_lines)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .alignment(Alignment::Left)
            .style(Style::default().bg(app.theme.bg_primary));

        f.render_widget(preview_text, chunks[1]);
    }
}

fn render_feed_menu(f: &mut Frame, app: &App) {
    let feeds = get_all_feeds();

    // Create centered popup
    let area = f.area();
    let popup_width = 60.min(area.width - 4);
    let popup_height = (feeds.len() as u16 + 4).min(area.height - 4);

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area
    f.render_widget(Clear, popup_area);

    // Create feed items
    let items: Vec<ListItem> = feeds
        .iter()
        .enumerate()
        .map(|(i, feed)| {
            let is_selected = i == app.feed_menu_selected;
            let is_current = feed.name == app.current_feed.name;

            let indicator = if is_current { "✓ " } else { "  " };
            let text = format!("{}{}", indicator, feed.name);

            let style = if is_selected {
                Style::default()
                    .fg(app.theme.accent_fg)
                    .bg(app.theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.fg_primary)
                    .bg(app.theme.bg_primary)
            };

            ListItem::new(Line::styled(text, style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title("Select Feed (f/Esc to close, Enter to select)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.accent))
            .style(Style::default().bg(app.theme.bg_primary)));

    let mut state = ListState::default();
    state.select(Some(app.feed_menu_selected));

    f.render_stateful_widget(list, popup_area, &mut state);
}

fn render_help_menu(f: &mut Frame, app: &App) {
    // Create centered popup
    let area = f.area();
    let popup_width = 70.min(area.width - 4);
    let popup_height = 22.min(area.height - 4);  // Enough for all help items + padding

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area
    f.render_widget(Clear, popup_area);

    // Create help content
    let help_text = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            "  j / ↓          Scroll down",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  k / ↑          Scroll up",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  G              Jump to bottom",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  l              Jump to top",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            "  o              Open article in browser",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  O              Open in new tab",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  Space          Jump to ticker article",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  r              Refresh news feed",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Views",
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            "  Tab            Toggle preview pane",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  a / Enter      Open article view",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  f              Open feed selector",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Settings",
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            "  s              Cycle sort order",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  t              Toggle date format",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  T              Cycle theme (light/dark)",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  p              Cycle image protocol",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
        )),
        Line::from(Span::styled(
            "  q / Esc        Quit",
            Style::default().fg(app.theme.fg_primary)
        )),
        Line::from(Span::styled(
            "  ?              Toggle this help menu",
            Style::default().fg(app.theme.fg_primary)
        )),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .title("Help (? or Esc to close)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.accent))
            .style(Style::default().bg(app.theme.bg_primary)))
        .alignment(Alignment::Left);

    f.render_widget(help_paragraph, popup_area);
}
