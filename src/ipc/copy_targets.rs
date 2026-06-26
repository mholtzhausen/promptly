use crate::config::AppConfig;
use crate::ipc::response::{cmd_err, cmd_ok};
use crate::ipc::types::{CopySettingsResult, CopyTargetDto, CopyTargetNamePayload};

pub fn cmd_get_copy_settings(id: &str) -> (String, bool, bool) {
    let config = AppConfig::load();
    let targets = config
        .sorted_copy_target_names()
        .into_iter()
        .filter_map(|name| {
            config.url_for_copy_target(&name).map(|url| CopyTargetDto {
                name,
                url,
            })
        })
        .collect::<Vec<_>>();
    let result = CopySettingsResult {
        targets,
        last_target: config.resolved_last_copy_target(),
    };
    cmd_ok(id, result)
}

pub fn cmd_set_last_copy_target(id: &str, payload: CopyTargetNamePayload) -> (String, bool, bool) {
    let mut config = AppConfig::load();
    if let Err(e) = config.set_last_copy_target(&payload.name) {
        return cmd_err(id, e);
    }
    if let Err(e) = config.save() {
        log::warn!("Failed to save copy target preference: {e}");
        return cmd_err(id, "Failed to save copy target preference");
    }
    cmd_ok(id, true)
}

pub fn cmd_open_copy_target(id: &str, payload: CopyTargetNamePayload) -> (String, bool, bool) {
    let mut config = AppConfig::load();
    let Some(url) = config.url_for_copy_target(&payload.name) else {
        return cmd_err(id, format!("Unknown copy target: {}", payload.name));
    };
    if let Err(e) = config.set_last_copy_target(&payload.name) {
        return cmd_err(id, e);
    }
    if let Err(e) = config.save() {
        log::warn!("Failed to save copy target preference: {e}");
        return cmd_err(id, "Failed to save copy target preference");
    }
    if let Err(e) = crate::browser::open_url_in_browser(&url) {
        log::warn!("Failed to open copy target URL: {e}");
        return cmd_err(id, "Failed to open URL in browser");
    }
    cmd_ok(id, true)
}

#[cfg(test)]
mod tests {
    use crate::ipc::test_support::{handle_raw, FakeEffects};
    use std::env;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn with_temp_config<F: FnOnce()>(f: F) {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("promptly").join("config.yml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        let prev_config_home = env::var("XDG_CONFIG_HOME").ok();
        env::set_var("XDG_CONFIG_HOME", dir.path());
        f();
        match prev_config_home {
            Some(value) => env::set_var("XDG_CONFIG_HOME", value),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn get_copy_settings_returns_defaults() {
        with_temp_config(|| {
            let (backend, _f) = crate::ipc::test_support::test_backend();
            let effects = FakeEffects::default();
            let handled = handle_raw(
                &backend,
                &effects,
                r#"{"id":"cs","command":"getCopySettings"}"#,
            );
            let resp = serde_json::from_str::<serde_json::Value>(&handled.response_json).unwrap();
            assert_eq!(resp["ok"], true);
            assert_eq!(resp["data"]["lastTarget"], "claude");
            let targets = resp["data"]["targets"].as_array().unwrap();
            assert!(targets.iter().any(|t| t["name"] == "claude"));
            assert!(targets.iter().any(|t| t["name"] == "gemini"));
        });
    }

    #[test]
    fn set_last_copy_target_persists() {
        with_temp_config(|| {
            let (backend, _f) = crate::ipc::test_support::test_backend();
            let effects = FakeEffects::default();
            let set = handle_raw(
                &backend,
                &effects,
                r#"{"id":"set","command":"setLastCopyTarget","payload":{"name":"gemini"}}"#,
            );
            let set_resp =
                serde_json::from_str::<serde_json::Value>(&set.response_json).unwrap();
            assert_eq!(set_resp["ok"], true);

            let get = handle_raw(
                &backend,
                &effects,
                r#"{"id":"get","command":"getCopySettings"}"#,
            );
            let get_resp =
                serde_json::from_str::<serde_json::Value>(&get.response_json).unwrap();
            assert_eq!(get_resp["data"]["lastTarget"], "gemini");
        });
    }

    #[test]
    fn open_copy_target_persists_last_target() {
        with_temp_config(|| {
            let (backend, _f) = crate::ipc::test_support::test_backend();
            let effects = FakeEffects::default();
            let open = handle_raw(
                &backend,
                &effects,
                r#"{"id":"open","command":"openCopyTarget","payload":{"name":"gemini"}}"#,
            );
            let open_resp =
                serde_json::from_str::<serde_json::Value>(&open.response_json).unwrap();
            assert_eq!(open_resp["ok"], true);

            let get = handle_raw(
                &backend,
                &effects,
                r#"{"id":"get","command":"getCopySettings"}"#,
            );
            let get_resp =
                serde_json::from_str::<serde_json::Value>(&get.response_json).unwrap();
            assert_eq!(get_resp["data"]["lastTarget"], "gemini");
        });
    }
}
