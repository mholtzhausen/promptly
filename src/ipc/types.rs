use crate::db;

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
    ListHistory,
    GetHistoryEntry(HistoryIdPayload),
    UpdateHistoryEntry(UpdateHistoryEntryPayload),
    DeleteHistoryEntry(HistoryIdPayload),
    PruneHistory(PruneHistoryPayload),
    SetWindowTitle(SetWindowTitlePayload),
    HideWindow,
    Quit,
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
    pub prompt_id: Option<i64>,
    pub values: Vec<VariableValue>,
    pub message_kind: CopyMessageKind,
    #[serde(default)]
    pub skip_history: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryIdPayload {
    pub id: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHistoryEntryPayload {
    pub id: i64,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PruneHistoryPayload {
    pub keep: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWindowTitlePayload {
    pub title: String,
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
    pub prompt: Option<db::Prompt>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyPromptResult {
    pub copied: bool,
    pub history_inserted: bool,
    pub history_count: i64,
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
    pub quit_app: bool,
    pub window_title: Option<String>,
}
