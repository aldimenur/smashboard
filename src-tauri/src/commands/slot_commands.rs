use std::path::Path;

use tauri::State;

use crate::audio::decoder::decode_audio;
use crate::models::slot::Slot;
use crate::AppState;

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
    slots.push(slot.clone());
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

    let slot = slots
        .iter_mut()
        .find(|item| item.id == slot_id)
        .ok_or_else(|| "slot not found".to_string())?;

    if let Some(value) = label {
        slot.label = value;
    }

    if let Some(value) = shortcut {
        slot.shortcut = value;
    }

    if let Some(value) = gain {
        slot.gain = value.clamp(0.0, 2.0);
    }

    Ok(slot.clone())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_slot(state: State<'_, AppState>, slot_id: String) -> Result<(), String> {
    let mut slots = state
        .slots
        .lock()
        .map_err(|_| "failed to lock slot state".to_string())?;

    let original_len = slots.len();
    slots.retain(|slot| slot.id != slot_id);

    if slots.len() == original_len {
        return Err("slot not found".to_string());
    }

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
pub async fn trigger_slot(state: State<'_, AppState>, slot_id: String) -> Result<(), String> {
    let slot = {
        let slots = state
            .slots
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
    let handle = state.audio_engine.play(buffer, slot.gain)?;
    tracing::debug!(
        playback_id = handle.id,
        duration_ms = handle.duration_ms,
        "slot triggered"
    );

    Ok(())
}
