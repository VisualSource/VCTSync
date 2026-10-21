use nanoserde::SerJson;

use crate::ipc_mappings::IpcError;

/// `{}` on success, `{ "error": ... }` otherwise.
pub fn ipc_reply(result: Result<(), String>) -> String {
    match result {
        Ok(()) => "{}".to_string(),
        Err(error) => IpcError { error }.serialize_json(),
    }
}
