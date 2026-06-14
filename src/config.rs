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
    border-radius: 8px;
    padding: 6px 10px;
    margin: 0;
}

entry#search-entry:focus {
    outline: 2px solid #C7D3DD;
    outline-offset: -2px;
}

listbox#prompt-list {
    background-color: transparent;
    margin: 0 6px;
}

listbox#prompt-list row {
    border-radius: 8px;
    margin: 2px 0;
}

listbox#prompt-list row:hover {
    background-color: #F0EDE5;
}

listbox#prompt-list row:selected {
    background-color: #E8E4DA;
}

.prompt-row {
    padding: 7px 8px;
}

.prompt-title {
    color: #1C1B18;
    font-weight: 700;
}

.prompt-description {
    color: #777167;
    font-size: 12px;
}

.prompt-actions button {
    border-radius: 6px;
    min-width: 28px;
    min-height: 28px;
    padding: 3px;
}

button#add-button, button#copy-button {
    background-color: #A3B9C9;
    color: #1a1a1a;
    border-radius: 8px;
    min-width: 34px;
    min-height: 34px;
    padding: 4px 10px;
    font-weight: bold;
}

button#add-button:hover, button#copy-button:hover {
    background-color: #8fa7b9;
}

/* Dialog buttons: smaller */
button#copy-button {
    min-height: 26px;
    padding: 2px 8px;
}

button#prompt-delete-confirm-button {
    background-color: #C98282;
    color: #1a1a1a;
    border-radius: 8px;
    padding: 6px 14px;
}

button#cancel-button {
    border-radius: 6px;
    padding: 4px 10px;
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

textview#template-textview text,
textview#prompt-preview-textview text {
    font-family: monospace;
    background-color: #FFFFFF;
    border-radius: 6px;
    padding: 8px;
}
"#;
