//! Typed IPC boundary between the React frontend and the Rust backend.
//!
//! The webview sends JSON requests via Wry's IPC channel; `IpcBackend::handle`
//! dispatches them against the existing `db` and `prompt_parser` modules and
//! returns a JSON response string.

use std::path::PathBuf;

use crate::db;
use crate::prompt_parser;

#[derive(Debug, serde::Deserialize)]
pub struct IpcRequest {
    pub id: String,
    #[serde(flatten)]
    pub command: IpcCommand,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "command", content = "payload", rename_all = "camelCase")]
pub enum IpcCommand {
    ListPrompts,
    SavePrompt(SavePromptPayload),
    DeletePrompt(DeletePromptPayload),
    VariablesForTemplate(TemplatePayload),
    Interpolate(InterpolatePayload),
    CopyPrompt(CopyPromptPayload),
    HideWindow,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePromptPayload {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePromptPayload {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct TemplatePayload {
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpolatePayload {
    pub template: String,
    pub values: Vec<VariableValue>,
}

#[derive(Debug, serde::Deserialize)]
pub struct VariableValue {
    pub name: String,
    pub value: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyPromptPayload {
    pub text: String,
    pub prompt_name: String,
    pub message_kind: CopyMessageKind,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CopyMessageKind {
    NoVariables,
    Variables,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcEnvelope<T: serde::Serialize> {
    pub id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePromptResult {
    pub saved: bool,
    pub prompt: Option<crate::db::Prompt>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDto {
    pub name: String,
    pub kind: String,
    pub default_value: String,
    pub description: String,
    pub options: Vec<String>,
}

/// Result of handling one IPC request: the JSON response to send back and
/// whether the webview window should be hidden afterwards.
pub struct HandledIpc {
    pub response_json: String,
    pub hide_window: bool,
}

/// Desktop side-effects the IPC layer needs: notifications and clipboard writes.
/// Implemented by `RealDesktopEffects` in production and fakes in tests.
pub trait DesktopEffects {
    fn notify(&self, summary: &str, body: &str);
    fn copy_text(&self, text: &str) -> anyhow::Result<()>;
}

/// Real desktop effects using `notify-rust` and `arboard`.
pub struct RealDesktopEffects;

impl DesktopEffects for RealDesktopEffects {
    fn notify(&self, summary: &str, body: &str) {
        use notify_rust::Notification;
        if let Err(e) = Notification::new().summary(summary).body(body).show() {
            log::error!("Failed to show notification: {}", e);
        }
    }

    fn copy_text(&self, text: &str) -> anyhow::Result<()> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(text.to_string())?;
        Ok(())
    }
}

pub struct IpcBackend {
    db_path: PathBuf,
}

impl IpcBackend {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Handle a raw JSON IPC request and produce the response + hide flag.
    pub fn handle(&self, raw: &str, effects: &impl DesktopEffects) -> HandledIpc {
        let request: IpcRequest = match serde_json::from_str(raw) {
            Ok(req) => req,
            Err(e) => {
                log::error!("Invalid IPC request: {}", e);
                let resp = IpcEnvelope::<serde_json::Value> {
                    id: String::new(),
                    ok: false,
                    data: None,
                    error: Some("Invalid IPC request".to_string()),
                };
                return HandledIpc {
                    response_json: serde_json::to_string(&resp).unwrap(),
                    hide_window: false,
                };
            }
        };

        let id = request.id.clone();
        let (response, hide_window) = self.dispatch(request.command, effects, &id);
        HandledIpc {
            response_json: response,
            hide_window,
        }
    }

    fn dispatch(
        &self,
        command: IpcCommand,
        effects: &impl DesktopEffects,
        id: &str,
    ) -> (String, bool) {
        match command {
            IpcCommand::ListPrompts => self.cmd_list_prompts(id),
            IpcCommand::SavePrompt(p) => self.cmd_save_prompt(id, p, effects),
            IpcCommand::DeletePrompt(p) => self.cmd_delete_prompt(id, p, effects),
            IpcCommand::VariablesForTemplate(p) => self.cmd_variables(id, p),
            IpcCommand::Interpolate(p) => self.cmd_interpolate(id, p),
            IpcCommand::CopyPrompt(p) => self.cmd_copy_prompt(id, p, effects),
            IpcCommand::HideWindow => {
                let resp = IpcEnvelope {
                    id: id.to_string(),
                    ok: true,
                    data: Some(true),
                    error: None,
                };
                (serde_json::to_string(&resp).unwrap(), true)
            }
        }
    }

    fn cmd_list_prompts(&self, id: &str) -> (String, bool) {
        match self.with_db(|conn| db::load_prompts(conn)) {
            Ok(prompts) => {
                let resp = IpcEnvelope {
                    id: id.to_string(),
                    ok: true,
                    data: Some(prompts),
                    error: None,
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
            Err(e) => {
                log::error!("listPrompts failed: {}", e);
                let resp = IpcEnvelope::<Vec<db::Prompt>> {
                    id: id.to_string(),
                    ok: false,
                    data: None,
                    error: Some(e.to_string()),
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
        }
    }

    fn cmd_save_prompt(
        &self,
        id: &str,
        p: SavePromptPayload,
        effects: &impl DesktopEffects,
    ) -> (String, bool) {
        let name = p.name.trim().to_string();
        let description = p.description.trim().to_string();
        let content = p.content;

        if name.is_empty() || description.is_empty() || content.trim().is_empty() {
            let resp = IpcEnvelope {
                id: id.to_string(),
                ok: true,
                data: Some(SavePromptResult {
                    saved: false,
                    prompt: None,
                }),
                error: None,
            };
            return (serde_json::to_string(&resp).unwrap(), false);
        }

        let result = self.with_db(|conn| {
            if let Some(existing_id) = p.id {
                db::update_prompt(conn, existing_id, &name, &description, &content)?;
                Ok(existing_id)
            } else {
                db::upsert_prompt(conn, &name, &description, &content).map(|id| id)
            }
        });

        match result {
            Ok(saved_id) => {
                // Reload the saved row by id.
                let prompt = self
                    .with_db(|conn| db::load_prompts(conn))
                    .ok()
                    .and_then(|prompts| prompts.into_iter().find(|pr| pr.id == saved_id));

                effects.notify(
                    "Prompt Saved",
                    &format!("Saved template '{}'", name),
                );

                let resp = IpcEnvelope {
                    id: id.to_string(),
                    ok: true,
                    data: Some(SavePromptResult {
                        saved: true,
                        prompt,
                    }),
                    error: None,
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
            Err(e) => {
                log::error!("savePrompt failed: {}", e);
                effects.notify(
                    "Prompt not saved",
                    "Could not save the prompt template.",
                );
                let resp = IpcEnvelope::<SavePromptResult> {
                    id: id.to_string(),
                    ok: false,
                    data: None,
                    error: Some(e.to_string()),
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
        }
    }

    fn cmd_delete_prompt(
        &self,
        id: &str,
        p: DeletePromptPayload,
        effects: &impl DesktopEffects,
    ) -> (String, bool) {
        match self.with_db(|conn| db::delete_prompt(conn, p.id)) {
            Ok(()) => {
                effects.notify(
                    "Prompt deleted",
                    &format!("Deleted template '{}'", p.name),
                );
                let resp = IpcEnvelope {
                    id: id.to_string(),
                    ok: true,
                    data: Some(true),
                    error: None,
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
            Err(e) => {
                log::error!("deletePrompt failed: {}", e);
                effects.notify(
                    "Prompt not deleted",
                    "Could not delete the prompt template.",
                );
                let resp = IpcEnvelope::<bool> {
                    id: id.to_string(),
                    ok: false,
                    data: None,
                    error: Some(e.to_string()),
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
        }
    }

    fn cmd_variables(&self, id: &str, p: TemplatePayload) -> (String, bool) {
        let vars = prompt_parser::parse_variables(&p.content);
        let dtos: Vec<VariableDto> = vars
            .into_iter()
            .map(|v| {
                let (kind, options) = match v.var_type {
                    prompt_parser::VarType::Text => ("text", Vec::new()),
                    prompt_parser::VarType::Number => ("number", Vec::new()),
                    prompt_parser::VarType::Multiline => ("multiline", Vec::new()),
                    prompt_parser::VarType::Option(opts) => ("option", opts),
                };
                VariableDto {
                    name: v.name,
                    kind: kind.to_string(),
                    default_value: v.default_value,
                    description: v.description,
                    options,
                }
            })
            .collect();

        let resp = IpcEnvelope {
            id: id.to_string(),
            ok: true,
            data: Some(dtos),
            error: None,
        };
        (serde_json::to_string(&resp).unwrap(), false)
    }

    fn cmd_interpolate(&self, id: &str, p: InterpolatePayload) -> (String, bool) {
        let pairs: Vec<(&str, &str)> = p
            .values
            .iter()
            .map(|v| (v.name.as_str(), v.value.as_str()))
            .collect();
        let result = prompt_parser::interpolate(&p.template, &pairs);

        let resp = IpcEnvelope {
            id: id.to_string(),
            ok: true,
            data: Some(result),
            error: None,
        };
        (serde_json::to_string(&resp).unwrap(), false)
    }

    fn cmd_copy_prompt(
        &self,
        id: &str,
        p: CopyPromptPayload,
        effects: &impl DesktopEffects,
    ) -> (String, bool) {
        match effects.copy_text(&p.text) {
            Ok(()) => {
                let body = match p.message_kind {
                    CopyMessageKind::NoVariables => {
                        format!("'{}' copied to clipboard", p.prompt_name)
                    }
                    CopyMessageKind::Variables => {
                        format!("'{}' copied to clipboard!", p.prompt_name)
                    }
                };
                effects.notify("Prompt copied", &body);
                let resp = IpcEnvelope {
                    id: id.to_string(),
                    ok: true,
                    data: Some(true),
                    error: None,
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
            Err(e) => {
                log::error!("copyPrompt failed: {}", e);
                effects.notify(
                    "Prompt not copied",
                    "Could not access the system clipboard.",
                );
                let resp = IpcEnvelope::<bool> {
                    id: id.to_string(),
                    ok: false,
                    data: None,
                    error: Some(e.to_string()),
                };
                (serde_json::to_string(&resp).unwrap(), false)
            }
        }
    }

    fn with_db<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&db::Connection) -> anyhow::Result<T>,
    {
        let conn = db::init_db(&self.db_path)?;
        f(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct FakeEffects {
        notifications: Rc<RefCell<Vec<(String, String)>>>,
        copied: Rc<RefCell<Vec<String>>>,
        copy_ok: bool,
    }

    impl DesktopEffects for FakeEffects {
        fn notify(&self, summary: &str, body: &str) {
            self.notifications
                .borrow_mut()
                .push((summary.to_string(), body.to_string()));
        }

        fn copy_text(&self, text: &str) -> anyhow::Result<()> {
            self.copied.borrow_mut().push(text.to_string());
            if self.copy_ok {
                Ok(())
            } else {
                anyhow::bail!("clipboard unavailable")
            }
        }
    }

    fn backend() -> (IpcBackend, tempfile::NamedTempFile) {
        let file = tempfile::NamedTempFile::new().unwrap();
        let backend = IpcBackend::new(file.path().to_path_buf());
        (backend, file)
    }

    fn handle(backend: &IpcBackend, effects: &FakeEffects, raw: &str) -> serde_json::Value {
        let handled = backend.handle(raw, effects);
        serde_json::from_str::<serde_json::Value>(&handled.response_json).unwrap()
    }

    #[test]
    fn save_prompt_trims_name_and_description_but_not_content() {
        let (backend, _f) = backend();
        let effects = FakeEffects::default();
        let raw = serde_json::json!({
            "id": "1",
            "command": "savePrompt",
            "payload": {
                "id": null,
                "name": "  git  ",
                "description": "  desc  ",
                "content": "  body  "
            }
        })
        .to_string();

        let resp = handle(&backend, &effects, &raw);
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"]["saved"], true);
        assert_eq!(resp["data"]["prompt"]["name"], "git");
        assert_eq!(resp["data"]["prompt"]["description"], "desc");
        assert_eq!(resp["data"]["prompt"]["content"], "  body  ");

        let notifs = effects.notifications.borrow();
        assert_eq!(notifs.len(), 1);
        assert_eq!(notifs[0].0, "Prompt Saved");
        assert_eq!(notifs[0].1, "Saved template 'git'");
    }

    #[test]
    fn save_prompt_validation_is_silent() {
        let (backend, _f) = backend();
        let effects = FakeEffects::default();
        let raw = serde_json::json!({
            "id": "2",
            "command": "savePrompt",
            "payload": {
                "id": null,
                "name": "x",
                "description": "   ",
                "content": "body"
            }
        })
        .to_string();

        let resp = handle(&backend, &effects, &raw);
        assert_eq!(resp["ok"], true);
        assert_eq!(resp["data"]["saved"], false);
        assert!(resp["data"]["prompt"].is_null());

        // No notification fired on validation failure.
        assert!(effects.notifications.borrow().is_empty());

        // Database remains empty.
        let list = handle(
            &backend,
            &effects,
            r#"{"id":"x","command":"listPrompts"}"#,
        );
        assert_eq!(list["data"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn variables_and_interpolation_reuse_parser() {
        let (backend, _f) = backend();
        let effects = FakeEffects::default();

        let vars_raw = serde_json::json!({
            "id": "3",
            "command": "variablesForTemplate",
            "payload": { "content": "Hello {{name|text|world|your name}}" }
        })
        .to_string();
        let resp = handle(&backend, &effects, &vars_raw);
        let var = &resp["data"][0];
        assert_eq!(var["name"], "name");
        assert_eq!(var["kind"], "text");
        assert_eq!(var["defaultValue"], "world");
        assert_eq!(var["description"], "your name");
        assert_eq!(var["options"].as_array().unwrap().len(), 0);

        let interp_raw = serde_json::json!({
            "id": "4",
            "command": "interpolate",
            "payload": {
                "template": "Hello {{name|text|world|your name}}",
                "values": [{ "name": "name", "value": "Alice" }]
            }
        })
        .to_string();
        let resp = handle(&backend, &effects, &interp_raw);
        assert_eq!(resp["data"], "Hello Alice");
    }

    #[test]
    fn copy_prompt_uses_existing_notification_literals() {
        let (backend, _f) = backend();
        let mut effects = FakeEffects::default();
        effects.copy_ok = true;

        let raw = serde_json::json!({
            "id": "5",
            "command": "copyPrompt",
            "payload": {
                "text": "Hello",
                "promptName": "plain-copy",
                "messageKind": "noVariables"
            }
        })
        .to_string();
        handle(&backend, &effects, &raw);
        assert_eq!(effects.copied.borrow()[0], "Hello");
        assert_eq!(
            effects.notifications.borrow()[0],
            ("Prompt copied".to_string(), "'plain-copy' copied to clipboard".to_string())
        );

        let raw = serde_json::json!({
            "id": "6",
            "command": "copyPrompt",
            "payload": {
                "text": "Hi",
                "promptName": "with-vars",
                "messageKind": "variables"
            }
        })
        .to_string();
        handle(&backend, &effects, &raw);
        assert_eq!(
            effects.notifications.borrow()[1],
            ("Prompt copied".to_string(), "'with-vars' copied to clipboard!".to_string())
        );
    }

    #[test]
    fn delete_prompt_removes_row_and_notifies() {
        let (backend, _f) = backend();
        let effects = FakeEffects::default();

        // Create a prompt first.
        let save_raw = serde_json::json!({
            "id": "7",
            "command": "savePrompt",
            "payload": {
                "id": null,
                "name": "to-delete",
                "description": "d",
                "content": "c"
            }
        })
        .to_string();
        let save_resp = handle(&backend, &effects, &save_raw);
        let saved_id = save_resp["data"]["prompt"]["id"].as_i64().unwrap();

        let del_raw = serde_json::json!({
            "id": "8",
            "command": "deletePrompt",
            "payload": { "id": saved_id, "name": "to-delete" }
        })
        .to_string();
        let del_resp = handle(&backend, &effects, &del_raw);
        assert_eq!(del_resp["ok"], true);
        assert_eq!(del_resp["data"], true);

        let notifs = effects.notifications.borrow();
        let delete_notif = notifs
            .iter()
            .find(|(s, _)| s == "Prompt deleted")
            .unwrap();
        assert_eq!(delete_notif.1, "Deleted template 'to-delete'");

        let list = handle(
            &backend,
            &effects,
            r#"{"id":"x","command":"listPrompts"}"#,
        );
        assert_eq!(list["data"].as_array().unwrap().len(), 0);
    }
}
