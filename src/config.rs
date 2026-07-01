//! Configuration: paths, constants, and design tokens.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const APP_NAME: &str = "promptly";

pub const DEFAULT_WINDOW_WIDTH: f64 = 500.0;
pub const DEFAULT_WINDOW_HEIGHT: f64 = 400.0;
pub const DEFAULT_EPHEMERAL_NOTIFICATION_SECONDS: u64 = 3;
pub const DEFAULT_LAST_COPY_TARGET: &str = "claude";

pub const ABOUT_WINDOW_WIDTH: f64 = 400.0;
pub const ABOUT_WINDOW_HEIGHT: f64 = 480.0;

pub const SETTINGS_WINDOW_WIDTH: f64 = 560.0;
pub const SETTINGS_WINDOW_HEIGHT: f64 = 420.0;

pub const RESERVED_CATEGORY_GENERAL: &str = "general";

const MIN_WINDOW_WIDTH: f64 = 320.0;
const MIN_WINDOW_HEIGHT: f64 = 240.0;

pub fn config_dir() -> PathBuf {
    let dir = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.join(APP_NAME)
}

pub fn config_file_path() -> PathBuf {
    config_dir().join("config.yml")
}

pub fn state_dir() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(path).join(APP_NAME);
    }
    dirs_next::home_dir()
        .unwrap_or_else(config_dir)
        .join(".local/state")
        .join(APP_NAME)
}

pub fn log_file_path() -> PathBuf {
    state_dir().join("promptly.log")
}

pub fn lock_file_path() -> PathBuf {
    config_dir().join("promptly.lock")
}

/// Override default DB path (used by tests and `PROMPTLY_DB_PATH`).
pub fn db_path() -> PathBuf {
    if let Ok(path) = std::env::var("PROMPTLY_DB_PATH") {
        return PathBuf::from(path);
    }
    config_dir().join("prompts.db")
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

fn default_ephemeral_notification_seconds() -> u64 {
    DEFAULT_EPHEMERAL_NOTIFICATION_SECONDS
}

pub fn default_copy_targets() -> HashMap<String, String> {
    HashMap::from([
        (
            "claude".to_string(),
            "https://claude.ai/new".to_string(),
        ),
        (
            "gemini".to_string(),
            "https://gemini.google.com/app".to_string(),
        ),
    ])
}

fn default_last_copy_target() -> String {
    DEFAULT_LAST_COPY_TARGET.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryDef {
    pub slug: String,
    pub label: String,
    pub chip_class: String,
}

pub fn default_categories() -> Vec<CategoryDef> {
    vec![
        CategoryDef {
            slug: "development".to_string(),
            label: "Development".to_string(),
            chip_class: "prompt-category--development".to_string(),
        },
        CategoryDef {
            slug: "agents".to_string(),
            label: "Agents".to_string(),
            chip_class: "prompt-category--agents".to_string(),
        },
        CategoryDef {
            slug: "communication".to_string(),
            label: "Communication".to_string(),
            chip_class: "prompt-category--communication".to_string(),
        },
        CategoryDef {
            slug: "writing".to_string(),
            label: "Writing".to_string(),
            chip_class: "prompt-category--writing".to_string(),
        },
        CategoryDef {
            slug: "image".to_string(),
            label: "Image".to_string(),
            chip_class: "prompt-category--image".to_string(),
        },
        CategoryDef {
            slug: RESERVED_CATEGORY_GENERAL.to_string(),
            label: "General".to_string(),
            chip_class: "prompt-category--general".to_string(),
        },
    ]
}

fn default_categories_vec() -> Vec<CategoryDef> {
    default_categories()
}

/// Validate a category slug for config storage.
pub fn validate_category_slug(slug: &str) -> Result<(), String> {
    let trimmed = slug.trim();
    if trimmed.is_empty() {
        return Err("Category slug cannot be empty".to_string());
    }
    if trimmed.len() > 64 {
        return Err("Category slug exceeds 64 characters".to_string());
    }
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return Err("Category slug cannot be empty".to_string());
    };
    if !first.is_ascii_lowercase() {
        return Err("Category slug must start with a lowercase letter".to_string());
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        return Err(
            "Category slug may only contain lowercase letters, digits, underscores, and hyphens"
                .to_string(),
        );
    }
    Ok(())
}

/// Validate a list of category definitions for save.
pub fn validate_categories(categories: &[CategoryDef]) -> Result<(), String> {
    if categories.is_empty() {
        return Err("At least one category is required".to_string());
    }
    let mut slugs = std::collections::HashSet::new();
    let mut has_general = false;
    for cat in categories {
        validate_category_slug(&cat.slug)?;
        if cat.label.trim().is_empty() {
            return Err(format!("Category '{}' label cannot be empty", cat.slug));
        }
        if cat.chip_class.trim().is_empty() {
            return Err(format!("Category '{}' chip class cannot be empty", cat.slug));
        }
        if !slugs.insert(cat.slug.clone()) {
            return Err(format!("Duplicate category slug: {}", cat.slug));
        }
        if cat.slug == RESERVED_CATEGORY_GENERAL {
            has_general = true;
        }
    }
    if !has_general {
        return Err(format!(
            "Category '{}' is required and cannot be removed",
            RESERVED_CATEGORY_GENERAL
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub window: Option<WindowSize>,
    #[serde(default = "default_ephemeral_notification_seconds")]
    pub ephemeral_notification_seconds: u64,
    #[serde(default = "default_categories_vec")]
    pub categories: Vec<CategoryDef>,
    #[serde(default = "default_copy_targets")]
    pub copy_targets: HashMap<String, String>,
    #[serde(default = "default_last_copy_target")]
    pub last_copy_target: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: None,
            ephemeral_notification_seconds: DEFAULT_EPHEMERAL_NOTIFICATION_SECONDS,
            categories: default_categories(),
            copy_targets: default_copy_targets(),
            last_copy_target: default_last_copy_target(),
        }
    }
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

    pub fn ephemeral_notification_ms(&self) -> u64 {
        self.ephemeral_notification_seconds.saturating_mul(1000)
    }

    pub fn effective_categories(&self) -> Vec<CategoryDef> {
        if self.categories.is_empty() {
            return default_categories();
        }
        self.categories.clone()
    }

    pub fn settings_window_size(&self) -> WindowSize {
        WindowSize {
            width: SETTINGS_WINDOW_WIDTH,
            height: SETTINGS_WINDOW_HEIGHT,
        }
    }

    pub fn effective_copy_targets(&self) -> HashMap<String, String> {
        if self.copy_targets.is_empty() {
            return default_copy_targets();
        }
        self.copy_targets.clone()
    }

    pub fn resolved_last_copy_target(&self) -> String {
        let targets = self.effective_copy_targets();
        if targets.contains_key(&self.last_copy_target) {
            return self.last_copy_target.clone();
        }
        let mut names: Vec<&String> = targets.keys().collect();
        names.sort();
        names
            .first()
            .map(|name| (*name).clone())
            .unwrap_or_else(|| DEFAULT_LAST_COPY_TARGET.to_string())
    }

    pub fn set_last_copy_target(&mut self, name: &str) -> Result<(), String> {
        if !self.effective_copy_targets().contains_key(name) {
            return Err(format!("Unknown copy target: {name}"));
        }
        self.last_copy_target = name.to_string();
        Ok(())
    }

    pub fn url_for_copy_target(&self, name: &str) -> Option<String> {
        self.effective_copy_targets().get(name).cloned()
    }

    pub fn sorted_copy_target_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.effective_copy_targets().into_keys().collect();
        names.sort();
        names
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
            ..AppConfig::default()
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
    fn default_config_uses_default_ephemeral_notification_seconds() {
        let config = AppConfig::default();
        assert_eq!(
            config.ephemeral_notification_seconds,
            DEFAULT_EPHEMERAL_NOTIFICATION_SECONDS
        );
        assert_eq!(config.ephemeral_notification_ms(), 3000);
    }

    #[test]
    fn yaml_roundtrip_preserves_ephemeral_notification_seconds() {
        let config = AppConfig {
            window: None,
            ephemeral_notification_seconds: 5,
            ..AppConfig::default()
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.ephemeral_notification_seconds, 5);
    }

    #[test]
    fn set_window_size_clamps_tiny_dimensions() {
        let mut config = AppConfig::default();
        config.set_window_size(10.0, 10.0);
        let size = config.window_size();
        assert_eq!(size.width, MIN_WINDOW_WIDTH);
        assert_eq!(size.height, MIN_WINDOW_HEIGHT);
    }

    #[test]
    fn default_config_includes_copy_targets() {
        let config = AppConfig::default();
        let targets = config.effective_copy_targets();
        assert_eq!(
            targets.get("claude").map(String::as_str),
            Some("https://claude.ai/new")
        );
        assert_eq!(
            targets.get("gemini").map(String::as_str),
            Some("https://gemini.google.com/app")
        );
        assert_eq!(config.resolved_last_copy_target(), "claude");
    }

    #[test]
    fn yaml_roundtrip_preserves_copy_targets() {
        let mut copy_targets = default_copy_targets();
        copy_targets.insert("custom".to_string(), "https://example.com".to_string());
        let config = AppConfig {
            window: None,
            ephemeral_notification_seconds: DEFAULT_EPHEMERAL_NOTIFICATION_SECONDS,
            copy_targets,
            last_copy_target: "gemini".to_string(),
            ..AppConfig::default()
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.resolved_last_copy_target(), "gemini");
        assert_eq!(
            parsed.url_for_copy_target("custom").as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn invalid_last_copy_target_falls_back_to_first_sorted() {
        let config = AppConfig {
            window: None,
            ephemeral_notification_seconds: DEFAULT_EPHEMERAL_NOTIFICATION_SECONDS,
            copy_targets: default_copy_targets(),
            last_copy_target: "missing".to_string(),
            ..AppConfig::default()
        };
        assert_eq!(config.resolved_last_copy_target(), "claude");
    }

    #[test]
    fn set_last_copy_target_rejects_unknown_name() {
        let mut config = AppConfig::default();
        assert!(config.set_last_copy_target("unknown").is_err());
        assert!(config.set_last_copy_target("claude").is_ok());
        assert_eq!(config.last_copy_target, "claude");
    }

    #[test]
    fn default_config_includes_categories_with_general() {
        let config = AppConfig::default();
        let categories = config.effective_categories();
        assert_eq!(categories.len(), 6);
        assert!(categories.iter().any(|c| c.slug == "general"));
    }

    #[test]
    fn yaml_roundtrip_preserves_categories() {
        let mut categories = default_categories();
        categories.push(CategoryDef {
            slug: "custom".to_string(),
            label: "Custom".to_string(),
            chip_class: "prompt-category--general".to_string(),
        });
        let config = AppConfig {
            categories,
            ..AppConfig::default()
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.effective_categories().len(), 7);
    }

    #[test]
    fn validate_categories_requires_general() {
        let mut categories = default_categories();
        categories.retain(|c| c.slug != "general");
        assert!(validate_categories(&categories).is_err());
    }

    #[test]
    fn validate_category_slug_rejects_invalid() {
        assert!(validate_category_slug("").is_err());
        assert!(validate_category_slug("Bad").is_err());
        assert!(validate_category_slug("ok-slug_2").is_ok());
    }
}
