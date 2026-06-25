//! Application metadata shown in the About pane.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

pub const FEATURES: &[&str] = &[
    "System tray access with global hotkey (Ctrl+Alt+Space)",
    "Fuzzy search across prompt templates",
    "Variable interpolation with type-aware inputs",
    "Live preview and clipboard copy with notifications",
    "Copy history with search, edit, and prune",
    "Create, edit, and delete prompt templates",
    "Persistent window size for the prompt manager",
    "GitHub release update checks from the tray menu",
];

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
}

pub fn app_info() -> AppInfo {
    AppInfo {
        version: VERSION.to_string(),
        description: DESCRIPTION.to_string(),
        features: FEATURES.iter().map(|s| (*s).to_string()).collect(),
    }
}
