use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::{api, article_fetcher, date_utils, feeds};

#[derive(Parser)]
#[command(name = "bbcli")]
#[command(about = "Browse BBC News like a hacker", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Specify feed name (e.g., world, technology, business)
    #[arg(short, long, global = true)]
    pub feed: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List headlines from feed
    List,

    /// Open article in browser by index
    Open {
        /// Article index (1-based)
        index: usize,
    },

    /// Show article in terminal
    Show {
        /// Article index (1-based)
        index: usize,
    },
}

pub fn run_cli(cli: Cli) -> Result<()> {
    // Get feed URL
    let feed = if let Some(feed_name) = &cli.feed {
        feeds::get_feed_by_name(feed_name)?
    } else {
        feeds::get_default_feed()
    };

    match cli.command {
        Some(Commands::List) => list_headlines(&feed),
        Some(Commands::Open { index }) => open_article(&feed, index),
        Some(Commands::Show { index }) => show_article(&feed, index),
        None => {
            // No subcommand provided, default to listing
            list_headlines(&feed)
        }
    }
}

fn list_headlines(feed: &feeds::Feed) -> Result<()> {
    let stories = api::fetch_stories(&feed.url)?;

    if stories.is_empty() {
        println!("No stories available.");
        return Ok(());
    }

    println!("# {}\n", feed.name);

    for (i, story) in stories.iter().enumerate() {
        let humanized_date = date_utils::humanize_time(&story.pub_date);
        println!("{}. {} ({})", i + 1, story.title, humanized_date);
    }

    Ok(())
}

fn open_article(feed: &feeds::Feed, index: usize) -> Result<()> {
    let stories = api::fetch_stories(&feed.url)?;

    if index == 0 || index > stories.len() {
        anyhow::bail!("Invalid article index: {}. Available: 1-{}", index, stories.len());
    }

    let story = &stories[index - 1];
    println!("Opening: {}", story.title);
    webbrowser::open(&story.link)?;
    println!("Launched in browser.");

    Ok(())
}

fn show_article(feed: &feeds::Feed, index: usize) -> Result<()> {
    let stories = api::fetch_stories(&feed.url)?;

    if index == 0 || index > stories.len() {
        anyhow::bail!("Invalid article index: {}. Available: 1-{}", index, stories.len());
    }

    let story = &stories[index - 1];

    // Fetch full article content
    let article_text = article_fetcher::fetch_article_content(&story.link)?;

    // Print to terminal
    println!("{}", article_text);

    Ok(())
}
