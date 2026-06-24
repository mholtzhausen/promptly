use super::types::IpcEnvelope;

/// JSON response string plus hide-window and quit-app flags.
pub type CmdResult = (String, bool, bool);

pub fn ok_json<T: serde::Serialize>(id: &str, data: T) -> String {
    let resp = IpcEnvelope {
        id: id.to_string(),
        ok: true,
        data: Some(data),
        error: None,
    };
    serde_json::to_string(&resp).unwrap()
}

pub fn err_json<T: serde::Serialize>(id: &str, error: impl std::fmt::Display) -> String {
    let resp = IpcEnvelope::<T> {
        id: id.to_string(),
        ok: false,
        data: None,
        error: Some(error.to_string()),
    };
    serde_json::to_string(&resp).unwrap()
}

pub fn cmd_ok<T: serde::Serialize>(id: &str, data: T) -> CmdResult {
    (ok_json(id, data), false, false)
}

pub fn cmd_err(id: &str, error: impl std::fmt::Display) -> CmdResult {
    (err_json::<serde_json::Value>(id, error), false, false)
}

pub fn invalid_request_json() -> String {
    err_json::<serde_json::Value>("", "Invalid IPC request")
}
