use crate::db;

use super::response::{cmd_err, cmd_ok};
use super::types::{HistoryIdPayload, PruneHistoryPayload, UpdateHistoryEntryPayload};
use super::IpcBackend;

impl IpcBackend {
    pub(super) fn cmd_list_history(&self, id: &str) -> super::response::CmdResult {
        match self.with_conn(db::list_history) {
            Ok(result) => cmd_ok(id, result),
            Err(e) => {
                log::error!("listHistory failed: {}", e);
                cmd_err(id, e)
            }
        }
    }

    pub(super) fn cmd_get_history_entry(
        &self,
        id: &str,
        p: HistoryIdPayload,
    ) -> super::response::CmdResult {
        match self.with_conn(|conn| db::get_history_entry(conn, p.id)) {
            Ok(entry) => cmd_ok(id, entry),
            Err(e) => {
                log::error!("getHistoryEntry failed: {}", e);
                (
                    super::response::err_json::<db::HistoryEntry>(id, e),
                    false,
                    false,
                )
            }
        }
    }

    pub(super) fn cmd_update_history_entry(
        &self,
        id: &str,
        p: UpdateHistoryEntryPayload,
    ) -> super::response::CmdResult {
        if let Err(msg) = super::limits::validate_copy_text(&p.content) {
            return (super::response::err_json::<bool>(id, msg), false, false);
        }
        match self.with_conn(|conn| db::update_history_entry(conn, p.id, &p.content)) {
            Ok(()) => cmd_ok(id, true),
            Err(e) => {
                log::error!("updateHistoryEntry failed: {}", e);
                (super::response::err_json::<bool>(id, e), false, false)
            }
        }
    }

    pub(super) fn cmd_delete_history_entry(
        &self,
        id: &str,
        p: HistoryIdPayload,
    ) -> super::response::CmdResult {
        match self.with_conn(|conn| db::delete_history_entry(conn, p.id)) {
            Ok(()) => cmd_ok(id, true),
            Err(e) => {
                log::error!("deleteHistoryEntry failed: {}", e);
                (super::response::err_json::<bool>(id, e), false, false)
            }
        }
    }

    pub(super) fn cmd_prune_history(
        &self,
        id: &str,
        p: PruneHistoryPayload,
    ) -> super::response::CmdResult {
        if let Err(msg) = super::limits::validate_prune_keep(p.keep) {
            return (super::response::err_json::<bool>(id, msg), false, false);
        }
        match self.with_conn(|conn| db::prune_history_keep_last(conn, p.keep)) {
            Ok(()) => cmd_ok(id, true),
            Err(e) => {
                log::error!("pruneHistory failed: {}", e);
                (super::response::err_json::<bool>(id, e), false, false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{handle, test_backend, FakeEffects};

    #[test]
    fn history_list_get_delete_and_prune() {
        let (backend, _f) = test_backend();
        let effects = FakeEffects {
            copy_ok: true,
            ..Default::default()
        };

        let copy_raw = serde_json::json!({
            "id": "h1",
            "command": "copyPrompt",
            "payload": {
                "text": "Saved prompt",
                "promptName": "tpl",
                "promptId": 1,
                "values": [{ "name": "topic", "value": "rust" }],
                "messageKind": "variables"
            }
        })
        .to_string();
        handle(&backend, &effects, &copy_raw);

        let list_resp = handle(&backend, &effects, r#"{"id":"h2","command":"listHistory"}"#);
        assert_eq!(list_resp["ok"], true);
        assert_eq!(list_resp["data"]["totalCount"], 1);
        let entry_id = list_resp["data"]["entries"][0]["id"].as_i64().unwrap();

        let get_resp = handle(
            &backend,
            &effects,
            &serde_json::json!({
                "id": "h3",
                "command": "getHistoryEntry",
                "payload": { "id": entry_id }
            })
            .to_string(),
        );
        assert_eq!(get_resp["data"]["content"], "Saved prompt");
        assert_eq!(get_resp["data"]["variables"][0]["name"], "topic");

        let del_resp = handle(
            &backend,
            &effects,
            &serde_json::json!({
                "id": "h4",
                "command": "deleteHistoryEntry",
                "payload": { "id": entry_id }
            })
            .to_string(),
        );
        assert_eq!(del_resp["data"], true);

        handle(&backend, &effects, &copy_raw);
        let prune_resp = handle(
            &backend,
            &effects,
            &serde_json::json!({
                "id": "h5",
                "command": "pruneHistory",
                "payload": { "keep": 0 }
            })
            .to_string(),
        );
        assert_eq!(prune_resp["data"], true);

        let list_after = handle(&backend, &effects, r#"{"id":"h6","command":"listHistory"}"#);
        assert_eq!(list_after["data"]["totalCount"], 0);
    }
}
