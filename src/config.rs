//! Configuration: paths, constants, and design tokens.

use std::path::PathBuf;

pub const APP_NAME: &str = "promptly";

pub fn config_dir() -> PathBuf {
    let dir = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.join(APP_NAME)
}

pub fn db_path() -> PathBuf {
    config_dir().join("prompts.db")
}

pub fn ensure_config_dir() -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    Ok(())
}

