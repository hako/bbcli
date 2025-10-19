use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    Light,
    Dark,
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::Light
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub bg_primary: Color,      // Main background
    pub bg_secondary: Color,    // Alternate background (for list items)
    pub bg_accent: Color,       // Header/footer background
    pub fg_primary: Color,      // Main text
    pub fg_secondary: Color,    // Secondary/muted text
    pub accent: Color,          // BBC red / selection color
    pub accent_fg: Color,       // Text color on accent background
}

impl Theme {
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            bg_primary: Color::Rgb(255, 254, 252),  // Off-white
            bg_secondary: Color::Rgb(255, 254, 252), // Same as primary
            bg_accent: Color::Rgb(230, 230, 227),    // Light gray for footer
            fg_primary: Color::Rgb(0, 0, 0),         // Black text
            fg_secondary: Color::Rgb(148, 148, 148), // Gray text
            accent: Color::Rgb(234, 68, 57),         // BBC red
            accent_fg: Color::Rgb(255, 255, 255),    // White on red
        }
    }

    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            bg_primary: Color::Rgb(0, 0, 0),         // Pure black
            bg_secondary: Color::Rgb(20, 20, 20),    // Slightly lighter black
            bg_accent: Color::Rgb(40, 40, 40),       // Dark gray for footer
            fg_primary: Color::Rgb(255, 255, 255),   // White text
            fg_secondary: Color::Rgb(150, 150, 150), // Light gray text
            accent: Color::Rgb(234, 68, 57),         // BBC red (same)
            accent_fg: Color::Rgb(255, 255, 255),    // White on red (same)
        }
    }

    pub fn from_name(name: &ThemeName) -> Self {
        match name {
            ThemeName::Light => Self::light(),
            ThemeName::Dark => Self::dark(),
        }
    }
}
