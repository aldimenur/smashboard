use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::models::recording::RecordingStatus;
use crate::models::timeline::TimelineEvent;
use crate::timeline::playback::PlaybackStatus;
use crate::undo::UndoAction;
use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingTimeUpdate {
    time_ms: f64,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn start_recording(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        playback.stop();
    }

    state
        .playback_loop_running
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = app_handle.emit("playback-status-updated", PlaybackStatus::Stopped);

    {
        let start_offset_ms = {
            let timeline = state
                .timeline_state
                .lock()
                .map_err(|_| "failed to lock timeline state".to_string())?;
            timeline.playhead_position_ms.max(0.0)
        };

        let mut engine = state
            .recording_engine
            .lock()
            .map_err(|_| "failed to lock recording engine".to_string())?;
        engine.start(start_offset_ms)?;
    }

    state
        .recording_timer_running
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let recording_engine = state.recording_engine.clone();
    let recording_timer_running = state.recording_timer_running.clone();
    let app_handle_clone = app_handle.clone();

    thread::spawn(move || {
        while recording_timer_running.load(std::sync::atomic::Ordering::SeqCst) {
            let (status, time_ms) = match recording_engine.lock() {
                Ok(engine) => (engine.status(), engine.get_current_time_ms()),
                Err(_) => break,
            };

            if matches!(status, RecordingStatus::Idle | RecordingStatus::Stopped) {
                break;
            }

            let _ = app_handle_clone.emit("recording-time-update", RecordingTimeUpdate { time_ms });

            thread::sleep(Duration::from_millis(16));
        }

        recording_timer_running.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn pause_recording(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .recording_engine
        .lock()
        .map_err(|_| "failed to lock recording engine".to_string())?;
    engine.pause()
}

#[tauri::command(rename_all = "camelCase")]
pub async fn resume_recording(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .recording_engine
        .lock()
        .map_err(|_| "failed to lock recording engine".to_string())?;
    engine.resume()
}

#[tauri::command(rename_all = "camelCase")]
pub async fn stop_recording(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Vec<TimelineEvent>, String> {
    state
        .recording_timer_running
        .store(false, std::sync::atomic::Ordering::SeqCst);

    let mut engine = state
        .recording_engine
        .lock()
        .map_err(|_| "failed to lock recording engine".to_string())?;
    let events = engine.stop()?;
    drop(engine);

    if !events.is_empty() {
        {
            let mut timeline = state
                .timeline_state
                .lock()
                .map_err(|_| "failed to lock timeline state".to_string())?;
            timeline.add_events(events.clone());
        }

        {
            let mut undo = state
                .undo_manager
                .lock()
                .map_err(|_| "failed to lock undo manager".to_string())?;
            undo.push(UndoAction::AddEvents(events.clone()));
        }

        state.mark_dirty()?;
    }

    let _ = app_handle.emit("timeline-updated", ());

    Ok(events)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_recording_status(state: State<'_, AppState>) -> Result<RecordingStatus, String> {
    let engine = state
        .recording_engine
        .lock()
        .map_err(|_| "failed to lock recording engine".to_string())?;

    Ok(engine.status())
}
