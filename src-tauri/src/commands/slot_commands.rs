use std::path::Path;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, State};

use crate::audio::decoder::decode_audio;
use crate::audio::engine::AudioEngine;
use crate::models::slot::Slot;
use crate::recording::engine::RecordingEngine;
use crate::{sync_shortcuts_for_slots, AppState};

#[tauri::command(rename_all = "camelCase")]
pub async fn add_slot(
    state: State<'_, AppState>,
    file_path: String,
    label: Option<String>,
) -> Result<Slot, String> {
    let mut slots = state
        .slots
        .lock()
        .map_err(|_| "failed to lock slot state".to_string())?;

    if slots.len() >= state.max_slots {
        return Err(format!("maximum slot count ({}) reached", state.max_slots));
    }

    let slot = Slot::new(file_path, label)?;
    let mut next_slots = slots.clone();
    next_slots.push(slot.clone());

    sync_shortcuts_for_slots(&state, &next_slots)?;

    *slots = next_slots;
    state.mark_dirty()?;

    Ok(slot)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_slot(
    state: State<'_, AppState>,
    slot_id: String,
    label: Option<String>,
    shortcut: Option<String>,
    gain: Option<f32>,
) -> Result<Slot, String> {
    let mut slots = state
        .slots
        .lock()
        .map_err(|_| "failed to lock slot state".to_string())?;

    let target_index = slots
        .iter()
        .position(|item| item.id == slot_id)
        .ok_or_else(|| "slot not found".to_string())?;

    let mut updated = slots[target_index].clone();

    if let Some(value) = label {
        updated.label = value;
    }

    if let Some(value) = shortcut {
        updated.shortcut = value.trim().to_string();
    }

    if let Some(value) = gain {
        updated.gain = value.clamp(0.0, 2.0);
    }

    let mut next_slots = slots.clone();
    next_slots[target_index] = updated.clone();

    sync_shortcuts_for_slots(&state, &next_slots)?;

    *slots = next_slots;
    state.mark_dirty()?;

    Ok(updated)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_slot(state: State<'_, AppState>, slot_id: String) -> Result<(), String> {
    let mut slots = state
        .slots
        .lock()
        .map_err(|_| "failed to lock slot state".to_string())?;

    let mut next_slots = slots.clone();
    let original_len = next_slots.len();
    next_slots.retain(|slot| slot.id != slot_id);

    if next_slots.len() == original_len {
        return Err("slot not found".to_string());
    }

    sync_shortcuts_for_slots(&state, &next_slots)?;

    *slots = next_slots;
    state.mark_dirty()?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_slots(state: State<'_, AppState>) -> Result<Vec<Slot>, String> {
    let slots = state
        .slots
        .lock()
        .map_err(|_| "failed to lock slot state".to_string())?;

    Ok(slots.clone())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn trigger_slot(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    slot_id: String,
) -> Result<(), String> {
    trigger_slot_with_shared(
        &state.slots,
        &state.audio_engine,
        &state.recording_engine,
        &app_handle,
        &slot_id,
    )
}

pub(crate) fn trigger_slot_with_shared(
    slots: &Arc<Mutex<Vec<Slot>>>,
    audio_engine: &Arc<AudioEngine>,
    recording_engine: &Arc<Mutex<RecordingEngine>>,
    app_handle: &AppHandle,
    slot_id: &str,
) -> Result<(), String> {
    let slot = {
        let slots = slots
            .lock()
            .map_err(|_| "failed to lock slot state".to_string())?;

        slots
            .iter()
            .find(|item| item.id == slot_id)
            .cloned()
            .ok_or_else(|| "slot not found".to_string())?
    };

    let audio_path = Path::new(&slot.audio_path);
    let buffer = decode_audio(audio_path)?;
    let handle = audio_engine.play(buffer, slot.gain)?;

    let captured_event = {
        let mut recording_engine = recording_engine
            .lock()
            .map_err(|_| "failed to lock recording engine".to_string())?;

        if recording_engine.is_recording() {
            Some(recording_engine.capture_event(&slot)?)
        } else {
            None
        }
    };

    if let Some(event) = captured_event {
        let _ = app_handle.emit("recording-event-captured", event);
    }

    let _ = app_handle.emit("slot-triggered", slot.id.clone());

    tracing::debug!(
        playback_id = handle.id,
        duration_ms = handle.duration_ms,
        slot_id = slot.id,
        "slot triggered"
    );

    Ok(())
}
