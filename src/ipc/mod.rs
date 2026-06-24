//! Typed IPC boundary between the React frontend and the Rust backend.

mod effects;
mod history;
mod limits;
mod prompts;
mod response;
mod types;

#[cfg(test)]
mod contract;
#[cfg(test)]
mod test_support;

pub use effects::{DesktopEffects, RealDesktopEffects};
pub use types::HandledIpc;

use crate::db;
use types::{IpcCommand, IpcRequest, SetWindowTitlePayload};

use response::{invalid_request_json, ok_json};

pub struct IpcBackend {
    conn: db::Connection,
}

impl IpcBackend {
    pub fn new(db_path: std::path::PathBuf) -> anyhow::Result<Self> {
        let conn = db::init_db(&db_path)?;
        Ok(Self { conn })
    }

    /// Handle a raw JSON IPC request and produce the response + side-effect flags.
    pub fn handle(&self, raw: &str, effects: &impl DesktopEffects) -> HandledIpc {
        let request: IpcRequest = match serde_json::from_str(raw) {
            Ok(req) => req,
            Err(e) => {
                log::error!("Invalid IPC request: {}", e);
                return HandledIpc {
                    response_json: invalid_request_json(),
                    hide_window: false,
                    quit_app: false,
                    window_title: None,
                };
            }
        };

        let id = request.id.clone();
        let (response, hide_window, quit_app, window_title) =
            self.dispatch(request.command, effects, &id);
        HandledIpc {
            response_json: response,
            hide_window,
            quit_app,
            window_title,
        }
    }

    fn dispatch(
        &self,
        command: IpcCommand,
        effects: &impl DesktopEffects,
        id: &str,
    ) -> (String, bool, bool, Option<String>) {
        match command {
            IpcCommand::ListPrompts => {
                let (response, hide_window, quit_app) = self.cmd_list_prompts(id);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::SavePrompt(p) => {
                let (response, hide_window, quit_app) = self.cmd_save_prompt(id, p, effects);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::DeletePrompt(p) => {
                let (response, hide_window, quit_app) = self.cmd_delete_prompt(id, p, effects);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::VariablesForTemplate(p) => {
                let (response, hide_window, quit_app) = self.cmd_variables(id, p);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::Interpolate(p) => {
                let (response, hide_window, quit_app) = self.cmd_interpolate(id, p);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::CopyPrompt(p) => {
                let (response, hide_window, quit_app) = self.cmd_copy_prompt(id, p, effects);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::ListHistory => {
                let (response, hide_window, quit_app) = self.cmd_list_history(id);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::GetHistoryEntry(p) => {
                let (response, hide_window, quit_app) = self.cmd_get_history_entry(id, p);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::UpdateHistoryEntry(p) => {
                let (response, hide_window, quit_app) = self.cmd_update_history_entry(id, p);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::DeleteHistoryEntry(p) => {
                let (response, hide_window, quit_app) = self.cmd_delete_history_entry(id, p);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::PruneHistory(p) => {
                let (response, hide_window, quit_app) = self.cmd_prune_history(id, p);
                (response, hide_window, quit_app, None)
            }
            IpcCommand::SetWindowTitle(p) => self.cmd_set_window_title(id, p),
            IpcCommand::HideWindow => self.cmd_hide_window(id),
            IpcCommand::Quit => self.cmd_quit(id),
        }
    }

    fn cmd_set_window_title(
        &self,
        id: &str,
        p: SetWindowTitlePayload,
    ) -> (String, bool, bool, Option<String>) {
        (ok_json(id, true), false, false, Some(p.title))
    }

    fn cmd_hide_window(&self, id: &str) -> (String, bool, bool, Option<String>) {
        (ok_json(id, true), true, false, None)
    }

    fn cmd_quit(&self, id: &str) -> (String, bool, bool, Option<String>) {
        (ok_json(id, true), false, true, None)
    }

    fn with_conn<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&db::Connection) -> anyhow::Result<T>,
    {
        f(&self.conn)
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{handle_raw, test_backend, FakeEffects};

    #[test]
    fn set_window_title_returns_title_for_host() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects::default();
        let raw = serde_json::json!({
            "id": "t1",
            "command": "setWindowTitle",
            "payload": { "title": "Promptly | Find a prompt" }
        })
        .to_string();

        let handled = handle_raw(&backend, &effects, &raw);
        assert_eq!(
            handled.window_title.as_deref(),
            Some("Promptly | Find a prompt")
        );
        let resp = serde_json::from_str::<serde_json::Value>(&handled.response_json).unwrap();
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"], true);
    }

    #[test]
    fn hide_window_sets_hide_flag() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects::default();
        let handled = handle_raw(&backend, &effects, r#"{"id":"h","command":"hideWindow"}"#);
        assert!(handled.hide_window);
        assert!(!handled.quit_app);
    }

    #[test]
    fn quit_sets_quit_flag() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects::default();
        let handled = handle_raw(&backend, &effects, r#"{"id":"q","command":"quit"}"#);
        assert!(!handled.hide_window);
        assert!(handled.quit_app);
    }
}
