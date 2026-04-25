pub mod audio;
pub mod autosave;
pub mod commands;
pub mod export;
pub mod models;
pub mod project;
pub mod recording;
pub mod timeline;
pub mod undo;

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use audio::engine::AudioEngine;
use autosave::AutosaveManager;
use chrono::{DateTime, Utc};
use models::project::{Project, ProjectSettings};
use recording::engine::RecordingEngine;
use recording::shortcut_manager::ShortcutManager;
use tauri::{Emitter, Manager};
use timeline::playback::PlaybackEngine;
use timeline::state::TimelineState;
use undo::UndoManager;

use crate::models::slot::Slot;

const MAX_SLOTS: usize = 25;

pub struct AppState {
    pub slots: Arc<Mutex<Vec<Slot>>>,
    pub audio_engine: Arc<AudioEngine>,
    pub recording_engine: Arc<Mutex<RecordingEngine>>,
    pub shortcut_manager: Arc<Mutex<ShortcutManager>>,
    pub timeline_state: Arc<Mutex<TimelineState>>,
    pub playback_engine: Arc<Mutex<PlaybackEngine>>,
    pub playback_loop_running: Arc<AtomicBool>,
    pub playback_triggered_event_ids: Arc<Mutex<HashSet<String>>>,
    pub recording_timer_running: Arc<AtomicBool>,
    pub project_settings: Arc<Mutex<ProjectSettings>>,
    pub project_name: Arc<Mutex<String>>,
    pub project_created_at: Arc<Mutex<DateTime<Utc>>>,
    pub project_modified_at: Arc<Mutex<DateTime<Utc>>>,
    pub current_project_path: Arc<Mutex<Option<String>>>,
    pub last_saved_at: Arc<Mutex<Option<DateTime<Utc>>>>,
    pub has_unsaved_changes: Arc<AtomicBool>,
    pub undo_manager: Arc<Mutex<UndoManager>>,
    pub autosave_manager: Arc<Mutex<AutosaveManager>>,
    pub max_slots: usize,
}

impl AppState {
    fn new() -> Result<Self, String> {
        let audio_engine = AudioEngine::new()?;
        let shortcut_manager = ShortcutManager::new()?;
        let now = Utc::now();

        Ok(Self {
            slots: Arc::new(Mutex::new(Vec::new())),
            audio_engine: Arc::new(audio_engine),
            recording_engine: Arc::new(Mutex::new(RecordingEngine::new())),
            shortcut_manager: Arc::new(Mutex::new(shortcut_manager)),
            timeline_state: Arc::new(Mutex::new(TimelineState::default())),
            playback_engine: Arc::new(Mutex::new(PlaybackEngine::new())),
            playback_loop_running: Arc::new(AtomicBool::new(false)),
            playback_triggered_event_ids: Arc::new(Mutex::new(HashSet::new())),
            recording_timer_running: Arc::new(AtomicBool::new(false)),
            project_settings: Arc::new(Mutex::new(ProjectSettings::default())),
            project_name: Arc::new(Mutex::new("Untitled".to_string())),
            project_created_at: Arc::new(Mutex::new(now)),
            project_modified_at: Arc::new(Mutex::new(now)),
            current_project_path: Arc::new(Mutex::new(None)),
            last_saved_at: Arc::new(Mutex::new(None)),
            has_unsaved_changes: Arc::new(AtomicBool::new(false)),
            undo_manager: Arc::new(Mutex::new(UndoManager::new(50))),
            autosave_manager: Arc::new(Mutex::new(AutosaveManager::new(120))),
            max_slots: MAX_SLOTS,
        })
    }

    pub fn mark_dirty(&self) -> Result<(), String> {
        let now = Utc::now();
        let mut modified_at = self
            .project_modified_at
            .lock()
            .map_err(|_| "failed to lock project modified time".to_string())?;
        *modified_at = now;
        self.has_unsaved_changes.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn mark_clean(&self, path: Option<String>) -> Result<(), String> {
        let now = Utc::now();

        {
            let mut modified_at = self
                .project_modified_at
                .lock()
                .map_err(|_| "failed to lock project modified time".to_string())?;
            *modified_at = now;
        }

        {
            let mut last_saved_at = self
                .last_saved_at
                .lock()
                .map_err(|_| "failed to lock project save state".to_string())?;
            *last_saved_at = Some(now);
        }

        if let Some(value) = path {
            let mut current_project_path = self
                .current_project_path
                .lock()
                .map_err(|_| "failed to lock project path".to_string())?;
            *current_project_path = Some(value);
        }

        self.has_unsaved_changes.store(false, Ordering::SeqCst);

        if let Ok(mut autosave_manager) = self.autosave_manager.lock() {
            autosave_manager.mark_saved();
        }

        Ok(())
    }

    pub fn snapshot_project(&self) -> Result<Project, String> {
        let slots = self
            .slots
            .lock()
            .map_err(|_| "failed to lock slots".to_string())?
            .clone();

        let (events, total_duration_ms) = {
            let timeline = self
                .timeline_state
                .lock()
                .map_err(|_| "failed to lock timeline".to_string())?;
            (timeline.events.clone(), timeline.total_duration_ms)
        };

        let settings = self
            .project_settings
            .lock()
            .map_err(|_| "failed to lock project settings".to_string())?
            .clone();

        let project_name = self
            .project_name
            .lock()
            .map_err(|_| "failed to lock project name".to_string())?
            .clone();

        let created_at = *self
            .project_created_at
            .lock()
            .map_err(|_| "failed to lock project created time".to_string())?;

        let modified_at = *self
            .project_modified_at
            .lock()
            .map_err(|_| "failed to lock project modified time".to_string())?;

        Ok(export::project_from_state(
            &project_name,
            created_at,
            modified_at,
            &settings,
            &slots,
            &events,
            total_duration_ms,
        ))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt().with_target(false).try_init();

    let app_state = AppState::new().expect("failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            let state = app.state::<AppState>();
            let slots = state.slots.clone();
            let audio_engine = state.audio_engine.clone();
            let recording_engine = state.recording_engine.clone();
            let shortcut_manager = state.shortcut_manager.clone();
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                let receiver = global_hotkey::GlobalHotKeyEvent::receiver();

                while let Ok(event) = receiver.recv() {
                    if event.state() != global_hotkey::HotKeyState::Pressed {
                        continue;
                    }

                    let slot_id: Option<String> = shortcut_manager
                        .lock()
                        .ok()
                        .and_then(|manager| manager.handle_shortcut(event.id()))
                        .map(|slot| slot.id);

                    let Some(slot_id) = slot_id else {
                        continue;
                    };

                    if let Err(err) = commands::slot_commands::trigger_slot_with_shared(
                        &slots,
                        &audio_engine,
                        &recording_engine,
                        &app_handle,
                        &slot_id,
                    ) {
                        tracing::error!(?err, "failed to trigger slot from global shortcut");
                    }
                }
            });

            let slots = state.slots.clone();
            let timeline_state = state.timeline_state.clone();
            let project_settings = state.project_settings.clone();
            let project_name = state.project_name.clone();
            let project_created_at = state.project_created_at.clone();
            let project_modified_at = state.project_modified_at.clone();
            let has_unsaved_changes = state.has_unsaved_changes.clone();
            let autosave_manager = state.autosave_manager.clone();
            let app_handle = app.handle().clone();

            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(10));

                if !has_unsaved_changes.load(Ordering::SeqCst) {
                    continue;
                }

                let should_autosave = autosave_manager
                    .lock()
                    .map(|manager| manager.should_autosave())
                    .unwrap_or(false);

                if !should_autosave {
                    continue;
                }

                let project = match snapshot_project_from_shared(
                    &slots,
                    &timeline_state,
                    &project_settings,
                    &project_name,
                    &project_created_at,
                    &project_modified_at,
                ) {
                    Ok(project) => project,
                    Err(err) => {
                        tracing::error!(?err, "failed to snapshot project for autosave");
                        continue;
                    }
                };

                let path = AutosaveManager::get_autosave_path();
                match project.save_to_file(&path) {
                    Ok(()) => {
                        if let Ok(mut manager) = autosave_manager.lock() {
                            manager.mark_saved();
                        }
                        let _ = app_handle.emit("autosave-completed", path.to_string_lossy().to_string());
                    }
                    Err(err) => {
                        tracing::error!(?err, "autosave failed");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::slot_commands::add_slot,
            commands::slot_commands::add_slot_at_position,
            commands::slot_commands::update_slot,
            commands::slot_commands::delete_slot,
            commands::slot_commands::get_all_slots,
            commands::slot_commands::trigger_slot,
            commands::recording_commands::start_recording,
            commands::recording_commands::pause_recording,
            commands::recording_commands::resume_recording,
            commands::recording_commands::stop_recording,
            commands::recording_commands::get_recording_status,
            commands::shortcut_commands::set_global_shortcuts_enabled,
            commands::shortcut_commands::get_global_shortcuts_enabled,
            commands::timeline_commands::get_timeline_events,
            commands::timeline_commands::add_timeline_event,
            commands::timeline_commands::update_event_time,
            commands::timeline_commands::update_event_times,
            commands::timeline_commands::delete_timeline_events,
            commands::timeline_commands::duplicate_events,
            commands::timeline_commands::play_timeline,
            commands::timeline_commands::pause_timeline,
            commands::timeline_commands::stop_timeline,
            commands::timeline_commands::seek_timeline,
            commands::timeline_commands::get_playback_status,
            commands::project_commands::save_project,
            commands::project_commands::load_project,
            commands::project_commands::validate_audio_paths,
            commands::project_commands::update_audio_path,
            commands::project_commands::autosave,
            commands::project_commands::get_autosave_path,
            commands::project_commands::get_project_state,
            commands::project_commands::check_autosave_recovery,
            commands::project_commands::force_quit_app,
            commands::project_commands::update_board_layout,
            commands::undo_commands::undo,
            commands::undo_commands::redo,
            commands::undo_commands::get_undo_redo_state,
            commands::export_commands::export_audio_wav,
            commands::export_commands::export_audio_mp3,
            commands::export_commands::export_timeline_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn snapshot_project_from_shared(
    slots: &Arc<Mutex<Vec<Slot>>>,
    timeline_state: &Arc<Mutex<TimelineState>>,
    project_settings: &Arc<Mutex<ProjectSettings>>,
    project_name: &Arc<Mutex<String>>,
    project_created_at: &Arc<Mutex<DateTime<Utc>>>,
    project_modified_at: &Arc<Mutex<DateTime<Utc>>>,
) -> Result<Project, String> {
    let slots = slots
        .lock()
        .map_err(|_| "failed to lock slots".to_string())?
        .clone();

    let (events, total_duration_ms) = {
        let timeline = timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline".to_string())?;
        (timeline.events.clone(), timeline.total_duration_ms)
    };

    let settings = project_settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?
        .clone();

    let project_name = project_name
        .lock()
        .map_err(|_| "failed to lock project name".to_string())?
        .clone();

    let created_at = *project_created_at
        .lock()
        .map_err(|_| "failed to lock created_at".to_string())?;

    let modified_at = *project_modified_at
        .lock()
        .map_err(|_| "failed to lock modified_at".to_string())?;

    Ok(export::project_from_state(
        &project_name,
        created_at,
        modified_at,
        &settings,
        &slots,
        &events,
        total_duration_ms,
    ))
}

pub(crate) fn sync_shortcuts_for_slots(state: &AppState, slots: &[Slot]) -> Result<(), String> {
    let mut manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| "failed to lock shortcut manager".to_string())?;

    manager.sync_slots(slots)
}

pub(crate) fn apply_loaded_project(state: &AppState, project: &Project, file_path: &Path) -> Result<(), String> {
    let normalized_slots = normalize_slot_positions(&project.slots, state.max_slots);

    {
        let mut slots = state
            .slots
            .lock()
            .map_err(|_| "failed to lock slots".to_string())?;
        *slots = normalized_slots.clone();
    }

    sync_shortcuts_for_slots(state, &normalized_slots)?;

    {
        let mut shortcut_manager = state
            .shortcut_manager
            .lock()
            .map_err(|_| "failed to lock shortcut manager".to_string())?;
        shortcut_manager.set_enabled(project.settings.global_shortcuts_enabled)?;
    }

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline".to_string())?;
        timeline.events = project.timeline.events.clone();
        timeline.total_duration_ms = project.timeline.total_duration_ms;
        timeline.playhead_position_ms = 0.0;
    }

    {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        playback.stop();
        playback.seek(0.0);
    }

    state.playback_loop_running.store(false, Ordering::SeqCst);
    state
        .playback_triggered_event_ids
        .lock()
        .map_err(|_| "failed to lock playback trigger state".to_string())?
        .clear();

    {
        let mut settings = state
            .project_settings
            .lock()
            .map_err(|_| "failed to lock project settings".to_string())?;
        *settings = project.settings.clone();
    }

    {
        let mut project_name = state
            .project_name
            .lock()
            .map_err(|_| "failed to lock project name".to_string())?;
        *project_name = project.project_name.clone();
    }

    {
        let mut created_at = state
            .project_created_at
            .lock()
            .map_err(|_| "failed to lock project created time".to_string())?;
        *created_at = project.created_at;
    }

    {
        let mut modified_at = state
            .project_modified_at
            .lock()
            .map_err(|_| "failed to lock project modified time".to_string())?;
        *modified_at = project.modified_at;
    }

    {
        let mut current_project_path = state
            .current_project_path
            .lock()
            .map_err(|_| "failed to lock project path".to_string())?;
        *current_project_path = Some(file_path.to_string_lossy().to_string());
    }

    {
        let mut last_saved_at = state
            .last_saved_at
            .lock()
            .map_err(|_| "failed to lock project save timestamp".to_string())?;
        *last_saved_at = Some(Utc::now());
    }

    if let Ok(mut undo_manager) = state.undo_manager.lock() {
        undo_manager.clear();
    }

    if let Ok(mut autosave_manager) = state.autosave_manager.lock() {
        autosave_manager.mark_saved();
    }

    state.has_unsaved_changes.store(false, Ordering::SeqCst);

    Ok(())
}

fn normalize_slot_positions(slots: &[Slot], max_slots: usize) -> Vec<Slot> {
    let mut assigned = std::collections::HashSet::new();
    let mut normalized = Vec::new();
    let mut next_fallback_position = 0usize;

    for slot in slots.iter().cloned() {
        if normalized.len() >= max_slots {
            break;
        }

        let mut next_slot = slot;
        let preferred = next_slot.position;

        let position = if preferred < max_slots && !assigned.contains(&preferred) {
            preferred
        } else {
            while next_fallback_position < max_slots && assigned.contains(&next_fallback_position) {
                next_fallback_position += 1;
            }
            if next_fallback_position >= max_slots {
                break;
            }
            next_fallback_position
        };

        next_slot.position = position;
        assigned.insert(position);
        normalized.push(next_slot);
    }

    normalized.sort_by_key(|slot| slot.position);
    normalized
}
