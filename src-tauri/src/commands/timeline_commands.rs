use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::audio::decoder::decode_audio;
use crate::models::slot::Slot;
use crate::models::timeline::TimelineEvent;
use crate::timeline::playback::PlaybackStatus;
use crate::undo::{EventTimeChange, UndoAction};
use crate::AppState;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTimeUpdate {
    pub event_id: String,
    pub new_time_ms: f64,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_timeline_events(state: State<'_, AppState>) -> Result<Vec<TimelineEvent>, String> {
    let timeline = state
        .timeline_state
        .lock()
        .map_err(|_| "failed to lock timeline state".to_string())?;

    Ok(timeline.events.clone())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn add_timeline_event(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    slot_id: String,
    time_ms: f64,
) -> Result<TimelineEvent, String> {
    let slot = find_slot(&state, &slot_id)?;

    let event = TimelineEvent {
        event_id: Uuid::new_v4().to_string(),
        time_ms: time_ms.max(0.0),
        slot_id: slot.id,
        audio_path: slot.audio_path,
        label: slot.label,
        shortcut: slot.shortcut,
        gain: slot.gain,
        duration_ms: slot.duration_ms,
    };

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        timeline.add_event(event.clone());
    }

    push_undo_action(&state, UndoAction::AddEvents(vec![event.clone()]))?;
    state.mark_dirty()?;
    emit_timeline_updated(&app_handle)?;

    Ok(event)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_event_time(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    event_id: String,
    new_time_ms: f64,
) -> Result<(), String> {
    let change = {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;

        let old_time_ms = timeline
            .events
            .iter()
            .find(|event| event.event_id == event_id)
            .map(|event| event.time_ms)
            .ok_or_else(|| "event not found".to_string())?;

        timeline.update_event_time(&event_id, new_time_ms.max(0.0));

        EventTimeChange {
            event_id,
            old_time_ms,
            new_time_ms,
        }
    };

    push_undo_action(&state, UndoAction::UpdateEventTimes(vec![change]))?;
    state.mark_dirty()?;
    emit_timeline_updated(&app_handle)?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_event_times(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    updates: Vec<EventTimeUpdate>,
) -> Result<(), String> {
    if updates.is_empty() {
        return Ok(());
    }

    let changes = {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;

        let mut changes = Vec::new();

        for update in updates {
            let old_time_ms = timeline
                .events
                .iter()
                .find(|event| event.event_id == update.event_id)
                .map(|event| event.time_ms)
                .ok_or_else(|| format!("event not found: {}", update.event_id))?;

            let next_time = update.new_time_ms.max(0.0);
            timeline.update_event_time(&update.event_id, next_time);

            changes.push(EventTimeChange {
                event_id: update.event_id,
                old_time_ms,
                new_time_ms: next_time,
            });
        }

        changes
    };

    push_undo_action(&state, UndoAction::UpdateEventTimes(changes))?;
    state.mark_dirty()?;
    emit_timeline_updated(&app_handle)?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_timeline_events(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    event_ids: Vec<String>,
) -> Result<(), String> {
    if event_ids.is_empty() {
        return Ok(());
    }

    let deleted = {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;

        let deleted = timeline
            .events
            .iter()
            .filter(|event| event_ids.contains(&event.event_id))
            .cloned()
            .collect::<Vec<_>>();

        timeline.delete_events(&event_ids);
        deleted
    };

    if deleted.is_empty() {
        return Ok(());
    }

    push_undo_action(&state, UndoAction::DeleteEvents(deleted))?;
    state.mark_dirty()?;
    emit_timeline_updated(&app_handle)?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn duplicate_events(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    event_ids: Vec<String>,
) -> Result<Vec<TimelineEvent>, String> {
    let duplicated = {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;

        let source_events = timeline
            .events
            .iter()
            .filter(|event| event_ids.contains(&event.event_id))
            .cloned()
            .collect::<Vec<_>>();

        let duplicated = source_events
            .into_iter()
            .map(|mut event| {
                event.event_id = Uuid::new_v4().to_string();
                event.time_ms += 33.33;
                event
            })
            .collect::<Vec<_>>();

        timeline.add_events(duplicated.clone());
        duplicated
    };

    if duplicated.is_empty() {
        return Ok(Vec::new());
    }

    push_undo_action(&state, UndoAction::AddEvents(duplicated.clone()))?;
    state.mark_dirty()?;
    emit_timeline_updated(&app_handle)?;

    Ok(duplicated)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn play_timeline(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    let from_time_ms = {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        let from_time = playback.get_current_time();
        playback.play(from_time);
        from_time
    };

    update_triggered_events_for_seek(&state, from_time_ms)?;
    let _ = app_handle.emit("playback-status-updated", PlaybackStatus::Playing);

    if state.playback_loop_running.load(Ordering::SeqCst) {
        return Ok(());
    }

    state.playback_loop_running.store(true, Ordering::SeqCst);

    let playback_loop_running = state.playback_loop_running.clone();
    let playback_engine = state.playback_engine.clone();
    let timeline_state = state.timeline_state.clone();
    let triggered_event_ids = state.playback_triggered_event_ids.clone();
    let audio_engine = state.audio_engine.clone();
    let app_handle_clone = app_handle.clone();

    thread::spawn(move || {
        while playback_loop_running.load(Ordering::SeqCst) {
            let current_time = {
                let playback = match playback_engine.lock() {
                    Ok(playback) => playback,
                    Err(_) => break,
                };

                if playback.status() != PlaybackStatus::Playing {
                    break;
                }

                playback.get_current_time()
            };

            let (events, total_duration_ms) = match timeline_state.lock() {
                Ok(mut timeline) => {
                    timeline.set_playhead_position(current_time);
                    let events = timeline.get_events_at_time(current_time, 100.0);
                    (events, timeline.total_duration_ms)
                }
                Err(_) => break,
            };

            for event in events {
                let should_trigger = {
                    let mut triggered = match triggered_event_ids.lock() {
                        Ok(triggered) => triggered,
                        Err(_) => continue,
                    };
                    triggered.insert(event.event_id.clone())
                };

                if !should_trigger {
                    continue;
                }

                if let Ok(buffer) = decode_audio(Path::new(&event.audio_path)) {
                    if let Err(err) = audio_engine.play(buffer, event.gain) {
                        tracing::debug!(?err, event_id = event.event_id, "failed to play timeline event");
                    }
                }

                let _ = app_handle_clone.emit("event-triggered", event.event_id.clone());
            }

            let _ = app_handle_clone.emit("playhead-update", current_time);

            if current_time >= total_duration_ms + 100.0 {
                if let Ok(mut playback) = playback_engine.lock() {
                    playback.stop();
                }
                let _ = app_handle_clone.emit("playback-status-updated", PlaybackStatus::Stopped);
                playback_loop_running.store(false, Ordering::SeqCst);
                break;
            }

            thread::sleep(Duration::from_millis(16));
        }

        playback_loop_running.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn pause_timeline(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        playback.pause();
    }

    state.playback_loop_running.store(false, Ordering::SeqCst);
    let _ = app_handle.emit("playback-status-updated", PlaybackStatus::Paused);

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn stop_timeline(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        playback.stop();
    }

    state.playback_loop_running.store(false, Ordering::SeqCst);
    let _ = app_handle.emit("playback-status-updated", PlaybackStatus::Stopped);

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn seek_timeline(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    time_ms: f64,
) -> Result<(), String> {
    let target_time = time_ms.max(0.0);

    {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        playback.seek(target_time);
    }

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        timeline.set_playhead_position(target_time);
    }

    update_triggered_events_for_seek(&state, target_time)?;

    let _ = app_handle.emit("playhead-update", target_time);

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_playback_status(state: State<'_, AppState>) -> Result<PlaybackStatus, String> {
    let playback = state
        .playback_engine
        .lock()
        .map_err(|_| "failed to lock playback engine".to_string())?;

    Ok(playback.status())
}

#[tauri::command]
pub async fn reset_timeline(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    {
        let mut playback = state
            .playback_engine
            .lock()
            .map_err(|_| "failed to lock playback engine".to_string())?;
        playback.stop();
        playback.seek(0.0);
    }
    let _ = state.audio_engine.stop_all();
    state.playback_loop_running.store(false, Ordering::SeqCst);

    state
        .playback_triggered_event_ids
        .lock()
        .map_err(|_| "failed to lock playback trigger state".to_string())?
        .clear();

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        timeline.events.clear();
        timeline.total_duration_ms = 0.0;
        timeline.playhead_position_ms = 0.0;
    }

    if let Ok(mut undo) = state.undo_manager.lock() {
        undo.clear();
    }

    state.mark_dirty()?;
    let _ = app_handle.emit("timeline-updated", ());
    let _ = app_handle.emit("playhead-update", 0.0f64);
    let _ = app_handle.emit("playback-status-updated", PlaybackStatus::Stopped);
    Ok(())
}

fn push_undo_action(state: &AppState, action: UndoAction) -> Result<(), String> {
    let mut undo = state
        .undo_manager
        .lock()
        .map_err(|_| "failed to lock undo manager".to_string())?;
    undo.push(action);
    Ok(())
}

fn emit_timeline_updated(app_handle: &AppHandle) -> Result<(), String> {
    app_handle
        .emit("timeline-updated", ())
        .map_err(|err| format!("failed to emit timeline update: {err}"))
}

fn find_slot(state: &AppState, slot_id: &str) -> Result<Slot, String> {
    let slots = state
        .slots
        .lock()
        .map_err(|_| "failed to lock slot state".to_string())?;

    slots
        .iter()
        .find(|slot| slot.id == slot_id)
        .cloned()
        .ok_or_else(|| "slot not found".to_string())
}

fn update_triggered_events_for_seek(state: &AppState, time_ms: f64) -> Result<(), String> {
    let event_ids_before_time = {
        let timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        timeline.get_events_before_time(time_ms)
    };

    let mut triggered = state
        .playback_triggered_event_ids
        .lock()
        .map_err(|_| "failed to lock playback trigger state".to_string())?;

    *triggered = event_ids_before_time.into_iter().collect::<HashSet<String>>();

    Ok(())
}
