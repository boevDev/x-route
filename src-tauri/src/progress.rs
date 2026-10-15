use serde::Serialize;
use tauri::Emitter;

pub const PROGRESS_EVENT: &str = "dns-progress";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent<'a> {
    stage: &'a str,
    operation: &'a str,
    detail: Option<&'a str>,
}

pub fn emit_progress(app: &tauri::AppHandle, stage: &str, operation: &str, detail: Option<&str>) {
    let _ = app.emit(
        PROGRESS_EVENT,
        ProgressEvent {
            stage,
            operation,
            detail,
        },
    );
}
