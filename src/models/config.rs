use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    TokyoNightDark,
    TokyoNightLight,
    CatppuccinMocha,
    CatppuccinLatte,
    DefaultDefault, // Fallback ke warna original Terminal
}

impl AppTheme {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TokyoNightDark => "TokyoNight (Dark)",
            Self::TokyoNightLight => "TokyoNight (Light)",
            Self::CatppuccinMocha => "Catppuccin Mocha (Dark)",
            Self::CatppuccinLatte => "Catppuccin Latte (Light)",
            Self::DefaultDefault => "Terminal Default",
        }
    }

    pub fn bg(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(26, 27, 38), // #1a1b26
            Self::TokyoNightLight => Color::Rgb(225, 226, 231), // #e1e2e7
            Self::CatppuccinMocha => Color::Rgb(30, 30, 46), // #1e1e2e
            Self::CatppuccinLatte => Color::Rgb(239, 241, 245), // #eff1f5
            Self::DefaultDefault => Color::Reset,
        }
    }

    pub fn unselected_bg(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(41, 46, 66), // #292e42
            Self::TokyoNightLight => Color::Rgb(203, 204, 214), // #cbccd6
            Self::CatppuccinMocha => Color::Rgb(49, 50, 68), // #313244
            Self::CatppuccinLatte => Color::Rgb(204, 208, 218), // #ccd0da
            Self::DefaultDefault => Color::DarkGray,
        }
    }

    pub fn text(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(192, 202, 245), // #c0caf5
            Self::TokyoNightLight => Color::Rgb(55, 96, 191),  // #3760bf
            Self::CatppuccinMocha => Color::Rgb(205, 214, 244), // #cdd6f4
            Self::CatppuccinLatte => Color::Rgb(76, 79, 105),  // #4c4f69
            Self::DefaultDefault => Color::White,
        }
    }

    pub fn muted_text(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(86, 95, 137), // #565f89
            Self::TokyoNightLight => Color::Rgb(132, 140, 181), // #848cb5
            Self::CatppuccinMocha => Color::Rgb(108, 112, 134), // #6c7086
            Self::CatppuccinLatte => Color::Rgb(140, 143, 161), // #8c8fa1
            Self::DefaultDefault => Color::DarkGray,
        }
    }

    pub fn primary(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(122, 162, 247), // #7aa2f7
            Self::TokyoNightLight => Color::Rgb(46, 125, 233), // #2e7de9
            Self::CatppuccinMocha => Color::Rgb(137, 180, 250), // #89b4fa
            Self::CatppuccinLatte => Color::Rgb(30, 102, 245), // #1e66f5
            Self::DefaultDefault => Color::Cyan,
        }
    }

    pub fn secondary(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(157, 124, 216), // #9d7cd8
            Self::TokyoNightLight => Color::Rgb(152, 84, 241), // #9854f1
            Self::CatppuccinMocha => Color::Rgb(245, 194, 231), // #f5c2e7
            Self::CatppuccinLatte => Color::Rgb(234, 118, 203), // #ea76cb
            Self::DefaultDefault => Color::Magenta,
        }
    }

    pub fn accent(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(255, 158, 100), // #ff9e64
            Self::TokyoNightLight => Color::Rgb(232, 102, 113), // #e86671
            Self::CatppuccinMocha => Color::Rgb(250, 179, 135), // #fab387
            Self::CatppuccinLatte => Color::Rgb(254, 100, 11), // #fe640b
            Self::DefaultDefault => Color::Yellow,
        }
    }

    pub fn safe(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(158, 206, 106), // #9ece6a
            Self::TokyoNightLight => Color::Rgb(88, 117, 57),  // #587539
            Self::CatppuccinMocha => Color::Rgb(166, 227, 161), // #a6e3a1
            Self::CatppuccinLatte => Color::Rgb(64, 160, 43),  // #40a02b
            Self::DefaultDefault => Color::Green,
        }
    }

    pub fn warning(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(224, 175, 104), // #e0af68
            Self::TokyoNightLight => Color::Rgb(140, 108, 62), // #8c6c3e
            Self::CatppuccinMocha => Color::Rgb(249, 226, 175), // #f9e2af
            Self::CatppuccinLatte => Color::Rgb(223, 142, 29), // #df8e1d
            Self::DefaultDefault => Color::Yellow,
        }
    }

    pub fn danger(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(247, 118, 142), // #f7768e
            Self::TokyoNightLight => Color::Rgb(245, 42, 101), // #f52a65
            Self::CatppuccinMocha => Color::Rgb(243, 139, 168), // #f38ba8
            Self::CatppuccinLatte => Color::Rgb(210, 15, 57),  // #d20f39
            Self::DefaultDefault => Color::Red,
        }
    }

    pub fn archive(&self) -> Color {
        match self {
            Self::TokyoNightDark => Color::Rgb(187, 154, 247), // #bb9af7
            Self::TokyoNightLight => Color::Rgb(152, 84, 241), // #9854f1
            Self::CatppuccinMocha => Color::Rgb(203, 166, 247), // #cba6f7
            Self::CatppuccinLatte => Color::Rgb(136, 57, 239), // #8839ef
            Self::DefaultDefault => Color::Blue,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::TokyoNightDark => Self::TokyoNightLight,
            Self::TokyoNightLight => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::CatppuccinLatte,
            Self::CatppuccinLatte => Self::DefaultDefault,
            Self::DefaultDefault => Self::TokyoNightDark,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::TokyoNightDark => Self::DefaultDefault,
            Self::TokyoNightLight => Self::TokyoNightDark,
            Self::CatppuccinMocha => Self::TokyoNightLight,
            Self::CatppuccinLatte => Self::CatppuccinMocha,
            Self::DefaultDefault => Self::CatppuccinLatte,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: AppTheme,
    pub archive_dir: PathBuf,
    pub safety_threshold_days: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        let user_dirs = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            theme: AppTheme::TokyoNightDark,
            archive_dir: user_dirs.join("Clario_Archives"),
            safety_threshold_days: 7,
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".config")
        });
        config_dir.join("clario").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}
