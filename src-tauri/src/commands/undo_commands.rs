use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::undo::UndoAction;
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRedoState {
    pub can_undo: bool,
    pub can_redo: bool,
}

#[tauri::command]
pub async fn undo(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    let action = state
        .undo_manager
        .lock()
        .map_err(|_| "failed to lock undo manager".to_string())?
        .undo()
        .ok_or_else(|| "nothing to undo".to_string())?;

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        apply_undo_action(&mut timeline, &action);
        timeline.recalculate_duration();
    }

    state.mark_dirty()?;

    let _ = app_handle.emit("timeline-updated", ());

    Ok(())
}

#[tauri::command]
pub async fn redo(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    let action = state
        .undo_manager
        .lock()
        .map_err(|_| "failed to lock undo manager".to_string())?
        .redo()
        .ok_or_else(|| "nothing to redo".to_string())?;

    {
        let mut timeline = state
            .timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline state".to_string())?;
        apply_redo_action(&mut timeline, &action);
        timeline.recalculate_duration();
    }

    state.mark_dirty()?;

    let _ = app_handle.emit("timeline-updated", ());

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_undo_redo_state(state: State<'_, AppState>) -> Result<UndoRedoState, String> {
    let manager = state
        .undo_manager
        .lock()
        .map_err(|_| "failed to lock undo manager".to_string())?;

    Ok(UndoRedoState {
        can_undo: manager.can_undo(),
        can_redo: manager.can_redo(),
    })
}

fn apply_undo_action(timeline: &mut crate::timeline::state::TimelineState, action: &UndoAction) {
    match action {
        UndoAction::AddEvents(events) => {
            let ids = events.iter().map(|event| event.event_id.clone()).collect::<Vec<_>>();
            timeline.delete_events(&ids);
        }
        UndoAction::DeleteEvents(events) => {
            timeline.add_events(events.clone());
        }
        UndoAction::UpdateEventTimes(changes) => {
            for change in changes {
                timeline.update_event_time(&change.event_id, change.old_time_ms);
            }
        }
    }
}

fn apply_redo_action(timeline: &mut crate::timeline::state::TimelineState, action: &UndoAction) {
    match action {
        UndoAction::AddEvents(events) => {
            timeline.add_events(events.clone());
        }
        UndoAction::DeleteEvents(events) => {
            let ids = events.iter().map(|event| event.event_id.clone()).collect::<Vec<_>>();
            timeline.delete_events(&ids);
        }
        UndoAction::UpdateEventTimes(changes) => {
            for change in changes {
                timeline.update_event_time(&change.event_id, change.new_time_ms);
            }
        }
    }
}
