use crate::db;
use crate::prompt_parser;

use super::effects::DesktopEffects;
use super::response::{cmd_err, cmd_ok};
use super::types::{
    CopyMessageKind, CopyPromptPayload, CopyPromptResult, DeletePromptPayload, InterpolatePayload,
    SavePromptPayload, SavePromptResult, TemplatePayload, VariableDto,
};
use super::IpcBackend;

impl IpcBackend {
    pub(super) fn cmd_list_prompts(&self, id: &str) -> super::response::CmdResult {
        match self.with_conn(db::load_prompts) {
            Ok(prompts) => cmd_ok(id, prompts),
            Err(e) => {
                log::error!("listPrompts failed: {}", e);
                cmd_err(id, e)
            }
        }
    }

    pub(super) fn cmd_save_prompt(
        &self,
        id: &str,
        p: SavePromptPayload,
        effects: &impl DesktopEffects,
    ) -> super::response::CmdResult {
        let name = p.name.trim().to_string();
        let description = p.description.trim().to_string();
        let content = p.content;

        if let Err(msg) = super::limits::validate_prompt_fields(&name, &description, &content) {
            return (
                super::response::err_json::<SavePromptResult>(id, msg),
                false,
                false,
            );
        }

        if name.is_empty() || description.is_empty() || content.trim().is_empty() {
            return cmd_ok(
                id,
                SavePromptResult {
                    saved: false,
                    prompt: None,
                },
            );
        }

        let result = self.with_conn(|conn| {
            if let Some(existing_id) = p.id {
                db::update_prompt(conn, existing_id, &name, &description, &content)?;
                Ok(existing_id)
            } else {
                db::upsert_prompt(conn, &name, &description, &content)
            }
        });

        match result {
            Ok(saved_id) => {
                let prompt = self
                    .with_conn(|conn| db::get_prompt_by_id(conn, saved_id))
                    .ok()
                    .flatten();

                effects.notify("Prompt Saved", &format!("Saved template '{name}'"));

                cmd_ok(
                    id,
                    SavePromptResult {
                        saved: true,
                        prompt,
                    },
                )
            }
            Err(e) => {
                log::error!("savePrompt failed: {}", e);
                effects.notify("Prompt not saved", "Could not save the prompt template.");
                (
                    super::response::err_json::<SavePromptResult>(id, e),
                    false,
                    false,
                )
            }
        }
    }

    pub(super) fn cmd_delete_prompt(
        &self,
        id: &str,
        p: DeletePromptPayload,
        effects: &impl DesktopEffects,
    ) -> super::response::CmdResult {
        match self.with_conn(|conn| db::delete_prompt(conn, p.id)) {
            Ok(()) => {
                effects.notify("Prompt deleted", &format!("Deleted template '{}'", p.name));
                cmd_ok(id, true)
            }
            Err(e) => {
                log::error!("deletePrompt failed: {}", e);
                effects.notify(
                    "Prompt not deleted",
                    "Could not delete the prompt template.",
                );
                (super::response::err_json::<bool>(id, e), false, false)
            }
        }
    }

    pub(super) fn cmd_variables(&self, id: &str, p: TemplatePayload) -> super::response::CmdResult {
        if let Err(msg) = super::limits::validate_template_content(&p.content) {
            return (
                super::response::err_json::<Vec<VariableDto>>(id, msg),
                false,
                false,
            );
        }
        let vars = prompt_parser::parse_variables(&p.content);
        if vars.len() > super::limits::MAX_VARIABLES_PER_TEMPLATE {
            return (
                super::response::err_json::<Vec<VariableDto>>(
                    id,
                    format!(
                        "Too many variables (max {})",
                        super::limits::MAX_VARIABLES_PER_TEMPLATE
                    ),
                ),
                false,
                false,
            );
        }
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

        cmd_ok(id, dtos)
    }

    pub(super) fn cmd_interpolate(
        &self,
        id: &str,
        p: InterpolatePayload,
    ) -> super::response::CmdResult {
        if let Err(msg) = super::limits::validate_template_content(&p.template) {
            return (super::response::err_json::<String>(id, msg), false, false);
        }
        let pairs_owned: Vec<(String, String)> = p
            .values
            .iter()
            .map(|v| (v.name.clone(), v.value.clone()))
            .collect();
        if let Err(msg) = super::limits::validate_interpolate_values(&pairs_owned) {
            return (super::response::err_json::<String>(id, msg), false, false);
        }
        let pairs: Vec<(&str, &str)> = pairs_owned
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        let result = prompt_parser::interpolate(&p.template, &pairs);
        cmd_ok(id, result)
    }

    pub(super) fn cmd_copy_prompt(
        &self,
        id: &str,
        p: CopyPromptPayload,
        effects: &impl DesktopEffects,
    ) -> super::response::CmdResult {
        if let Err(msg) = super::limits::validate_copy_text(&p.text) {
            return (
                super::response::err_json::<CopyPromptResult>(id, msg),
                false,
                false,
            );
        }
        let pairs: Vec<(String, String)> = p
            .values
            .iter()
            .map(|v| (v.name.clone(), v.value.clone()))
            .collect();
        if let Err(msg) = super::limits::validate_interpolate_values(&pairs) {
            return (
                super::response::err_json::<CopyPromptResult>(id, msg),
                false,
                false,
            );
        }
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

                let history_values: Vec<db::HistoryVariable> = p
                    .values
                    .iter()
                    .map(|v| db::HistoryVariable {
                        name: v.name.clone(),
                        value: v.value.clone(),
                    })
                    .collect();

                let (history_inserted, history_count) = if p.skip_history {
                    (false, 0)
                } else {
                    match self.with_conn(|conn| {
                        db::insert_history_if_new(
                            conn,
                            &p.text,
                            p.prompt_id,
                            &p.prompt_name,
                            &history_values,
                        )
                    }) {
                        Ok(result) => result,
                        Err(e) => {
                            log::error!("copyPrompt history insert failed: {}", e);
                            (false, 0)
                        }
                    }
                };

                cmd_ok(
                    id,
                    CopyPromptResult {
                        copied: true,
                        history_inserted,
                        history_count,
                    },
                )
            }
            Err(e) => {
                log::error!("copyPrompt failed: {}", e);
                effects.notify(
                    "Prompt not copied",
                    "Could not access the system clipboard.",
                );
                (
                    super::response::err_json::<CopyPromptResult>(id, e),
                    false,
                    false,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{handle, test_backend, FakeEffects};

    #[test]
    fn save_prompt_trims_name_and_description_but_not_content() {
        let (backend, _f) = test_backend();
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
        let (backend, _f) = test_backend();
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
        assert!(effects.notifications.borrow().is_empty());

        let list = handle(&backend, &effects, r#"{"id":"x","command":"listPrompts"}"#);
        assert_eq!(list["data"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn save_prompt_updates_existing_row() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects::default();

        let create = serde_json::json!({
            "id": "u1",
            "command": "savePrompt",
            "payload": {
                "id": null,
                "name": "orig",
                "description": "d1",
                "content": "c1"
            }
        })
        .to_string();
        let created = handle(&backend, &effects, &create);
        let saved_id = created["data"]["prompt"]["id"].as_i64().unwrap();

        let update = serde_json::json!({
            "id": "u2",
            "command": "savePrompt",
            "payload": {
                "id": saved_id,
                "name": "updated",
                "description": "d2",
                "content": "c2"
            }
        })
        .to_string();
        let updated = handle(&backend, &effects, &update);
        assert_eq!(updated["data"]["saved"], true);
        assert_eq!(updated["data"]["prompt"]["name"], "updated");
        assert_eq!(updated["data"]["prompt"]["id"], saved_id);
    }

    #[test]
    fn variables_and_interpolation_reuse_parser() {
        let (backend, _f) = test_backend();
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
        let (backend, _f) = test_backend();
        let effects = FakeEffects {
            copy_ok: true,
            ..Default::default()
        };

        let raw = serde_json::json!({
            "id": "5",
            "command": "copyPrompt",
            "payload": {
                "text": "Hello",
                "promptName": "plain-copy",
                "promptId": null,
                "values": [],
                "messageKind": "noVariables"
            }
        })
        .to_string();
        let resp = handle(&backend, &effects, &raw);
        assert_eq!(effects.copied.borrow()[0], "Hello");
        assert_eq!(
            effects.notifications.borrow()[0],
            (
                "Prompt copied".to_string(),
                "'plain-copy' copied to clipboard".to_string()
            )
        );
        assert_eq!(resp["data"]["copied"], true);
        assert_eq!(resp["data"]["historyInserted"], true);
        assert_eq!(resp["data"]["historyCount"], 1);

        let raw = serde_json::json!({
            "id": "6",
            "command": "copyPrompt",
            "payload": {
                "text": "Hi",
                "promptName": "with-vars",
                "promptId": 42,
                "values": [{ "name": "x", "value": "y" }],
                "messageKind": "variables"
            }
        })
        .to_string();
        handle(&backend, &effects, &raw);
        assert_eq!(
            effects.notifications.borrow()[1],
            (
                "Prompt copied".to_string(),
                "'with-vars' copied to clipboard!".to_string()
            )
        );

        let raw = serde_json::json!({
            "id": "6b",
            "command": "copyPrompt",
            "payload": {
                "text": "Hello",
                "promptName": "plain-copy",
                "promptId": null,
                "values": [],
                "messageKind": "noVariables"
            }
        })
        .to_string();
        let dup_resp = handle(&backend, &effects, &raw);
        assert_eq!(dup_resp["data"]["historyInserted"], false);
        assert_eq!(dup_resp["data"]["historyCount"], 2);
    }

    #[test]
    fn copy_prompt_skip_history() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects {
            copy_ok: true,
            ..Default::default()
        };

        let raw = serde_json::json!({
            "id": "sk1",
            "command": "copyPrompt",
            "payload": {
                "text": "No history",
                "promptName": "skip",
                "promptId": null,
                "values": [],
                "messageKind": "noVariables",
                "skipHistory": true
            }
        })
        .to_string();
        let resp = handle(&backend, &effects, &raw);
        assert_eq!(resp["data"]["historyInserted"], false);
        assert_eq!(resp["data"]["historyCount"], 0);

        let list = handle(
            &backend,
            &effects,
            r#"{"id":"sk2","command":"listHistory"}"#,
        );
        assert_eq!(list["data"]["totalCount"], 0);
    }

    #[test]
    fn delete_prompt_removes_row_and_notifies() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects::default();

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
        let delete_notif = notifs.iter().find(|(s, _)| s == "Prompt deleted").unwrap();
        assert_eq!(delete_notif.1, "Deleted template 'to-delete'");

        let list = handle(&backend, &effects, r#"{"id":"x","command":"listPrompts"}"#);
        assert_eq!(list["data"].as_array().unwrap().len(), 0);
    }
}
