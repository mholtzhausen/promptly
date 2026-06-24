//! Configuration: paths, constants, and design tokens.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const APP_NAME: &str = "promptly";

pub const DEFAULT_WINDOW_WIDTH: f64 = 500.0;
pub const DEFAULT_WINDOW_HEIGHT: f64 = 400.0;

const MIN_WINDOW_WIDTH: f64 = 320.0;
const MIN_WINDOW_HEIGHT: f64 = 240.0;

pub fn config_dir() -> PathBuf {
    let dir = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.join(APP_NAME)
}

pub fn db_path() -> PathBuf {
    config_dir().join("prompts.db")
}

pub fn config_file_path() -> PathBuf {
    config_dir().join("config.yml")
}

pub fn ensure_config_dir() -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

impl WindowSize {
    pub fn default_size() -> Self {
        Self {
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        }
    }

    fn sanitized(self) -> Self {
        Self {
            width: self.width.max(MIN_WINDOW_WIDTH),
            height: self.height.max(MIN_WINDOW_HEIGHT),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub window: Option<WindowSize>,
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_file_path();
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match serde_yaml::from_str(&raw) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Failed to parse {}: {e}", path.display());
                Self::default()
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        ensure_config_dir()?;
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(config_file_path(), yaml)?;
        Ok(())
    }

    pub fn window_size(&self) -> WindowSize {
        self.window
            .unwrap_or_else(WindowSize::default_size)
            .sanitized()
    }

    pub fn set_window_size(&mut self, width: f64, height: f64) {
        self.window = Some(WindowSize { width, height }.sanitized());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_default_window_size() {
        let config = AppConfig::default();
        let size = config.window_size();
        assert_eq!(size, WindowSize::default_size());
    }

    #[test]
    fn yaml_roundtrip_preserves_window_size() {
        let config = AppConfig {
            window: Some(WindowSize {
                width: 640.0,
                height: 480.0,
            }),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(
            parsed.window_size(),
            WindowSize {
                width: 640.0,
                height: 480.0,
            }
        );
    }

    #[test]
    fn set_window_size_clamps_tiny_dimensions() {
        let mut config = AppConfig::default();
        config.set_window_size(10.0, 10.0);
        let size = config.window_size();
        assert_eq!(size.width, MIN_WINDOW_WIDTH);
        assert_eq!(size.height, MIN_WINDOW_HEIGHT);
    }
}
