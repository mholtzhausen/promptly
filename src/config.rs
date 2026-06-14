//! Configuration: paths, constants, and design tokens.

use std::path::PathBuf;

pub const APP_NAME: &str = "prompt_tray";

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

pub const CSS: &str = r#"
* {
    font-family: sans-serif;
    font-size: 13px;
    line-height: 1.4;
}

window#popup-window {
    background-color: #F9F7F3;
}

entry#search-entry {
    background-color: #FFFFFF;
    border-radius: 6px;
    padding: 8px 12px;
    margin: 8px;
}

entry#search-entry:focus {
    outline: 2px solid #C7D3DD;
    outline-offset: -2px;
}

listbox#prompt-list row:hover {
    background-color: #F0EDE5;
}

listbox#prompt-list row:selected {
    background-color: #E8E4DA;
}

button#add-button, button#copy-button {
    background-color: #A3B9C9;
    color: #1a1a1a;
    border-radius: 6px;
    padding: 8px 16px;
    font-weight: bold;
}

button#add-button:hover, button#copy-button:hover {
    background-color: #8fa7b9;
}

button#cancel-button {
    border-radius: 6px;
    padding: 8px 16px;
}

entry.variable-entry, textview.variable-textview text {
    background-color: #FFFFFF;
    border-radius: 6px;
    padding: 6px 10px;
}

label#status-label {
    color: #B0AFA7;
    font-size: 11px;
    margin: 4px 8px 8px;
}

label.variable-label {
    font-weight: bold;
    margin-bottom: 2px;
}

label.variable-description {
    color: #B0AFA7;
    font-size: 11px;
}

textview#template-textview text {
    font-family: monospace;
    background-color: #FFFFFF;
    border-radius: 6px;
    padding: 8px;
}
"#;
