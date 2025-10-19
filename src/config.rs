use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::theme::ThemeName;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub theme: ThemeName,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyBindings {
    #[serde(default = "default_quit")]
    pub quit: char,
    #[serde(default = "default_open")]
    pub open: char,
    #[serde(default = "default_open_new_tab")]
    pub open_new_tab: char,
    #[serde(default = "default_refresh")]
    pub refresh: char,
    #[serde(default = "default_latest")]
    pub latest: char,
    #[serde(default = "default_scroll_up")]
    pub scroll_up: char,
    #[serde(default = "default_scroll_down")]
    pub scroll_down: char,
    #[serde(default = "default_scroll_bottom")]
    pub scroll_bottom: char,
}

fn default_quit() -> char { 'q' }
fn default_open() -> char { 'o' }
fn default_open_new_tab() -> char { 'O' }
fn default_refresh() -> char { 'r' }
fn default_latest() -> char { 'l' }
fn default_scroll_up() -> char { 'k' }
fn default_scroll_down() -> char { 'j' }
fn default_scroll_bottom() -> char { 'G' }

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: default_quit(),
            open: default_open(),
            open_new_tab: default_open_new_tab(),
            refresh: default_refresh(),
            latest: default_latest(),
            scroll_up: default_scroll_up(),
            scroll_down: default_scroll_down(),
            scroll_bottom: default_scroll_bottom(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybindings: KeyBindings::default(),
            theme: ThemeName::default(),
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&config_path)
        .context("Failed to read config file")?;

    let config: Config = toml::from_str(&content)
        .context("Failed to parse config file")?;

    Ok(config)
}

fn get_config_path() -> Result<PathBuf> {
    // Try ~/.bbcli first
    if let Some(home) = dirs::home_dir() {
        let bbcli_path = home.join(".bbcli");
        if bbcli_path.exists() {
            return Ok(bbcli_path);
        }
    }

    // Try ~/.config/bbcli
    if let Some(config_dir) = dirs::config_dir() {
        let bbcli_config = config_dir.join("bbcli");
        return Ok(bbcli_config);
    }

    // Fallback to home directory
    dirs::home_dir()
        .map(|h| h.join(".bbcli"))
        .context("Could not determine config path")
}
