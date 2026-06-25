//! IPC JSON contract fixtures shared with the React frontend types.

use super::types::{CopyPromptResult, SavePromptResult, VariableDto};
use crate::db::{HistoryEntry, HistoryListItem, HistoryListResult, Prompt};

#[test]
fn prompt_roundtrip_matches_frontend_shape() {
    let sample = Prompt {
        id: 1,
        name: "git".to_string(),
        description: "commit helper".to_string(),
        content: r#"fix: <var name="msg" type="text" />"#.to_string(),
    };
    let json = serde_json::to_value(&sample).unwrap();
    assert_eq!(json["id"], 1);
    assert_eq!(json["name"], "git");
    assert_eq!(json["description"], "commit helper");
    assert_eq!(json["content"], r#"fix: <var name="msg" type="text" />"#);
}

#[test]
fn variable_dto_roundtrip_matches_frontend_shape() {
    let dto = VariableDto {
        name: "msg".to_string(),
        kind: "text".to_string(),
        default_value: String::new(),
        label: "message".to_string(),
        placeholder: String::new(),
        options: vec![],
    };
    let json = serde_json::to_value(&dto).unwrap();
    assert_eq!(json["kind"], "text");
    assert_eq!(json["defaultValue"], "");
    assert_eq!(json["options"], serde_json::json!([]));
}

#[test]
fn save_and_copy_results_use_camel_case() {
    let save = SavePromptResult {
        saved: true,
        prompt: None,
    };
    let save_json = serde_json::to_value(&save).unwrap();
    assert!(save_json.get("saved").is_some());
    assert!(save_json.get("prompt").is_some());

    let copy = CopyPromptResult {
        copied: true,
        history_inserted: false,
        history_count: 3,
    };
    let copy_json = serde_json::to_value(&copy).unwrap();
    assert_eq!(copy_json["historyInserted"], false);
    assert_eq!(copy_json["historyCount"], 3);
}

#[test]
fn history_types_use_camel_case() {
    let list = HistoryListResult {
        entries: vec![HistoryListItem {
            id: 9,
            title: "[tpl](a:b)".to_string(),
            created_at: 100,
        }],
        total_count: 1,
    };
    let list_json = serde_json::to_value(&list).unwrap();
    assert_eq!(list_json["totalCount"], 1);
    assert_eq!(list_json["entries"][0]["createdAt"], 100);

    let entry = HistoryEntry {
        id: 9,
        title: "[tpl](a:b)".to_string(),
        content: "body".to_string(),
        variables: vec![crate::db::HistoryVariable {
            name: "a".to_string(),
            value: "b".to_string(),
        }],
        prompt_id: Some(1),
        prompt_name: "tpl".to_string(),
        created_at: 100,
    };
    let entry_json = serde_json::to_value(&entry).unwrap();
    assert_eq!(entry_json["promptId"], 1);
    assert_eq!(entry_json["promptName"], "tpl");
    assert_eq!(entry_json["createdAt"], 100);
}

#[test]
fn ipc_request_deserializes_frontend_envelope() {
    let raw = r#"{"id":"abc","command":"savePrompt","payload":{"id":null,"name":"n","description":"d","content":"c"}}"#;
    let req: super::types::IpcRequest = serde_json::from_str(raw).unwrap();
    assert_eq!(req.id, "abc");
    match req.command {
        super::types::IpcCommand::SavePrompt(p) => {
            assert_eq!(p.name, "n");
            assert!(p.id.is_none());
        }
        _ => panic!("expected savePrompt"),
    }
}
