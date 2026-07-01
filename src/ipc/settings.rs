use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::config::{
    validate_categories, AppConfig, CategoryDef, RESERVED_CATEGORY_GENERAL,
};
use crate::db;
use crate::ipc::response::{cmd_err, cmd_ok};
use crate::ipc::types::{
    AppSettingsResult, CategoryDefDto, CopyTargetDto, SaveAppSettingsPayload,
};

type SettingsCmdResult = (String, bool, bool, bool);

fn ok_flag<T: serde::Serialize>(id: &str, data: T, flag: bool) -> SettingsCmdResult {
    let (response, hide_window, quit_app) = cmd_ok(id, data);
    (response, hide_window, quit_app, flag)
}

fn err_flag(id: &str, error: impl std::fmt::Display) -> SettingsCmdResult {
    let (response, hide_window, quit_app) = cmd_err(id, error);
    (response, hide_window, quit_app, false)
}

pub fn cmd_get_app_settings(id: &str, config: &RefCell<AppConfig>) -> (String, bool, bool) {
    let config = config.borrow();
    let result = build_settings_result(&config);
    cmd_ok(id, result)
}

pub fn cmd_save_app_settings(
    id: &str,
    payload: SaveAppSettingsPayload,
    config: &Rc<RefCell<AppConfig>>,
    conn: &db::Connection,
) -> SettingsCmdResult {
    let mut config = config.borrow_mut();

    if let Some(seconds) = payload.ephemeral_notification_seconds {
        if !(1..=60).contains(&seconds) {
            return err_flag(
                id,
                "Notification timeout must be between 1 and 60 seconds",
            );
        }
        config.ephemeral_notification_seconds = seconds;
    }

    if let Some(categories) = payload.categories {
        if let Err(e) = apply_category_changes(conn, &config, &categories) {
            return err_flag(id, e);
        }
        let defs: Vec<CategoryDef> = categories
            .into_iter()
            .map(|c| CategoryDef {
                slug: c.slug,
                label: c.label,
                chip_class: c.chip_class,
            })
            .collect();
        if let Err(e) = validate_categories(&defs) {
            return err_flag(id, e);
        }
        config.categories = defs;
    }

    if let Some(targets) = payload.targets {
        if let Err(e) = apply_copy_targets(&mut config, targets) {
            return err_flag(id, e);
        }
    }

    if let Some(name) = payload.last_copy_target {
        if let Err(e) = config.set_last_copy_target(&name) {
            return err_flag(id, e);
        }
    }

    if let Err(e) = config.save() {
        log::warn!("Failed to save app settings: {e}");
        return err_flag(id, "Failed to save settings to config file");
    }

    let result = build_settings_result(&config);
    ok_flag(id, result, true)
}

pub fn cmd_open_settings_window(id: &str) -> SettingsCmdResult {
    ok_flag(id, true, true)
}

pub fn cmd_close_settings_window(id: &str) -> SettingsCmdResult {
    ok_flag(id, true, true)
}

fn build_settings_result(config: &AppConfig) -> AppSettingsResult {
    let categories = config
        .effective_categories()
        .into_iter()
        .map(|c| CategoryDefDto {
            slug: c.slug,
            label: c.label,
            chip_class: c.chip_class,
            previous_slug: None,
        })
        .collect();
    let targets = config
        .sorted_copy_target_names()
        .into_iter()
        .filter_map(|name| {
            config.url_for_copy_target(&name).map(|url| CopyTargetDto {
                name,
                url,
            })
        })
        .collect();
    AppSettingsResult {
        ephemeral_notification_seconds: config.ephemeral_notification_seconds,
        categories,
        targets,
        last_target: config.resolved_last_copy_target(),
    }
}

fn apply_category_changes(
    conn: &db::Connection,
    config: &AppConfig,
    categories: &[CategoryDefDto],
) -> Result<(), String> {
    let old_slugs: HashSet<String> = config
        .effective_categories()
        .into_iter()
        .map(|c| c.slug)
        .collect();

    for cat in categories {
        if let Some(prev) = cat.previous_slug.as_deref() {
            if prev != cat.slug {
                if prev == RESERVED_CATEGORY_GENERAL {
                    return Err("Cannot rename the General category".to_string());
                }
                db::rename_category(conn, prev, &cat.slug)
                    .map_err(|e| format!("Failed to rename category: {e}"))?;
            }
        }
    }

    let new_slugs: HashSet<String> = categories.iter().map(|c| c.slug.clone()).collect();
    for old_slug in &old_slugs {
        if !new_slugs.contains(old_slug) && old_slug != RESERVED_CATEGORY_GENERAL {
            db::reassign_category(conn, old_slug, RESERVED_CATEGORY_GENERAL)
                .map_err(|e| format!("Failed to reassign prompts: {e}"))?;
        }
    }

    Ok(())
}

fn apply_copy_targets(
    config: &mut AppConfig,
    targets: Vec<CopyTargetDto>,
) -> Result<(), String> {
    if targets.is_empty() {
        return Err("At least one copy target is required".to_string());
    }
    let mut map = HashMap::new();
    for target in targets {
        let name = target.name.trim();
        if name.is_empty() {
            return Err("Copy target name cannot be empty".to_string());
        }
        let url = target.url.trim();
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(format!(
                "Copy target '{name}' URL must start with http:// or https://"
            ));
        }
        if map.insert(name.to_string(), url.to_string()).is_some() {
            return Err(format!("Duplicate copy target name: {name}"));
        }
    }
    if !map.contains_key(&config.resolved_last_copy_target()) {
        let mut names: Vec<&String> = map.keys().collect();
        names.sort();
        if let Some(first) = names.first() {
            config.last_copy_target = (*first).clone();
        }
    }
    config.copy_targets = map;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::ipc::test_support::{handle_raw, test_backend, FakeEffects};
    use std::env;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn with_temp_config<F: FnOnce()>(f: F) {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("promptly")).unwrap();
        let prev_config_home = env::var("XDG_CONFIG_HOME").ok();
        env::set_var("XDG_CONFIG_HOME", dir.path());
        f();
        match prev_config_home {
            Some(value) => env::set_var("XDG_CONFIG_HOME", value),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn get_app_settings_returns_defaults() {
        with_temp_config(|| {
            let (backend, _f) = test_backend();
            let effects = FakeEffects::default();
            let handled = handle_raw(
                &backend,
                &effects,
                r#"{"id":"gs","command":"getAppSettings"}"#,
            );
            let resp = serde_json::from_str::<serde_json::Value>(&handled.response_json).unwrap();
            assert_eq!(resp["ok"], true);
            assert_eq!(resp["data"]["ephemeralNotificationSeconds"], 3);
            let categories = resp["data"]["categories"].as_array().unwrap();
            assert!(categories.iter().any(|c| c["slug"] == "general"));
        });
    }

    #[test]
    fn save_app_settings_persists_notification_timeout() {
        with_temp_config(|| {
            let (backend, _f) = test_backend();
            let effects = FakeEffects::default();
            let save = handle_raw(
                &backend,
                &effects,
                r#"{"id":"save","command":"saveAppSettings","payload":{"ephemeralNotificationSeconds":5}}"#,
            );
            let save_resp =
                serde_json::from_str::<serde_json::Value>(&save.response_json).unwrap();
            assert_eq!(save_resp["ok"], true);
            assert!(save.config_changed);

            let loaded = AppConfig::load();
            assert_eq!(loaded.ephemeral_notification_seconds, 5);
        });
    }
}
