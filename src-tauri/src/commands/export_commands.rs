use std::path::Path;

use tauri::State;

use crate::export::{export_timeline_to_json, export_timeline_to_mp3, export_timeline_to_wav};
use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn export_audio_wav(
    state: State<'_, AppState>,
    output_path: String,
    allow_missing_files: Option<bool>,
) -> Result<(), String> {
    let events = {
        let timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        timeline.events.clone()
    };

    export_timeline_to_wav(
        &events,
        Path::new(&output_path),
        allow_missing_files.unwrap_or(false),
    )
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_audio_mp3(
    state: State<'_, AppState>,
    output_path: String,
    allow_missing_files: Option<bool>,
) -> Result<(), String> {
    let events = {
        let timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        timeline.events.clone()
    };

    export_timeline_to_mp3(
        &events,
        Path::new(&output_path),
        allow_missing_files.unwrap_or(false),
    )
}

#[tauri::command(rename_all = "camelCase")]
pub async fn export_timeline_json(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<(), String> {
    let project = state.snapshot_project()?;
    export_timeline_to_json(&project, Path::new(&output_path))
}
