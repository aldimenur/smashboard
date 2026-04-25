use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::autosave::AutosaveManager;
use crate::models::project::Project;
use crate::{apply_loaded_project, AppState};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatePayload {
    pub project_name: String,
    pub current_path: Option<String>,
    pub has_unsaved_changes: bool,
    pub global_shortcuts_enabled: bool,
    pub frame_rate: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutosaveRecoveryInfo {
    pub has_recoverable: bool,
    pub autosave_path: String,
    pub modified_at: Option<String>,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn save_project(state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    let path = Path::new(&file_path);

    let mut project = state.snapshot_project()?;
    project.modified_at = Utc::now();

    if project.project_name == "Untitled" {
        if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
            project.project_name = stem.to_string();
        }
    }

    project.save_to_file(path)?;

    {
        let mut project_name = state
            .project_name
            .lock()
            .map_err(|_| "failed to lock project name".to_string())?;
        *project_name = project.project_name;
    }

    state.mark_clean(Some(file_path))?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn load_project(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    file_path: String,
) -> Result<Project, String> {
    let path = Path::new(&file_path);
    let project = Project::load_from_file(path)?;

    if project.slots.len() > state.max_slots {
        return Err(format!(
            "project has {} slots but current limit is {}",
            project.slots.len(),
            state.max_slots
        ));
    }

    apply_loaded_project(&state, &project, path)?;

    let _ = app_handle.emit("timeline-updated", ());
    let _ = app_handle.emit("project-loaded", project.clone());

    Ok(project)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn validate_audio_paths(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let project = state.snapshot_project()?;

    let mut missing = project.validate_audio_paths();

    for event in project.timeline.events {
        if !Path::new(&event.audio_path).exists() {
            missing.push(event.audio_path);
        }
    }

    missing.sort();
    missing.dedup();

    Ok(missing)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_audio_path(
    state: State<'_, AppState>,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    {
        let mut slots = state
            .slots
            .lock()
            .map_err(|_| "failed to lock slots".to_string())?;

        for slot in slots.iter_mut() {
            if slot.audio_path == old_path {
                slot.audio_path = new_path.clone();
            }
        }
    }

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline".to_string())?;

        for event in timeline.events.iter_mut() {
            if event.audio_path == old_path {
                event.audio_path = new_path.clone();
            }
        }
    }

    state.mark_dirty()?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn autosave(state: State<'_, AppState>) -> Result<(), String> {
    if !state
        .has_unsaved_changes
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Ok(());
    }

    let project = state.snapshot_project()?;
    let path = AutosaveManager::get_autosave_path();
    project.save_to_file(&path)?;

    if let Ok(mut manager) = state.autosave_manager.lock() {
        manager.mark_saved();
    }

    Ok(())
}

#[tauri::command]
pub async fn get_autosave_path() -> Result<String, String> {
    Ok(AutosaveManager::get_autosave_path()
        .to_string_lossy()
        .to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_project_state(state: State<'_, AppState>) -> Result<ProjectStatePayload, String> {
    let project_name = state
        .project_name
        .lock()
        .map_err(|_| "failed to lock project name".to_string())?
        .clone();

    let current_path = state
        .current_project_path
        .lock()
        .map_err(|_| "failed to lock project path".to_string())?
        .clone();

    let settings = state
        .project_settings
        .lock()
        .map_err(|_| "failed to lock project settings".to_string())?
        .clone();

    Ok(ProjectStatePayload {
        project_name,
        current_path,
        has_unsaved_changes: state
            .has_unsaved_changes
            .load(std::sync::atomic::Ordering::SeqCst),
        global_shortcuts_enabled: settings.global_shortcuts_enabled,
        frame_rate: settings.frame_rate,
    })
}

#[tauri::command(rename_all = "camelCase")]
pub async fn check_autosave_recovery(
    state: State<'_, AppState>,
) -> Result<AutosaveRecoveryInfo, String> {
    let autosave_path = AutosaveManager::get_autosave_path();
    let autosave_path_str = autosave_path.to_string_lossy().to_string();

    if !autosave_path.exists() {
        return Ok(AutosaveRecoveryInfo {
            has_recoverable: false,
            autosave_path: autosave_path_str,
            modified_at: None,
        });
    }

    let autosave_modified = std::fs::metadata(&autosave_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .map(DateTime::<Utc>::from);

    let last_saved_at = state
        .last_saved_at
        .lock()
        .map_err(|_| "failed to lock project save timestamp".to_string())?
        .to_owned();

    let has_recoverable = match (autosave_modified, last_saved_at) {
        (Some(autosave_time), Some(last_saved)) => autosave_time > last_saved,
        (Some(_), None) => true,
        _ => false,
    };

    Ok(AutosaveRecoveryInfo {
        has_recoverable,
        autosave_path: autosave_path_str,
        modified_at: autosave_modified.map(|value| value.to_rfc3339()),
    })
}
