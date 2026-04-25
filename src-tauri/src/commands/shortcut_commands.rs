use tauri::State;

use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn set_global_shortcuts_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut manager = state
            .shortcut_manager
            .lock()
            .map_err(|_| "failed to lock shortcut manager".to_string())?;

        manager.set_enabled(enabled)?;
    }

    {
        let mut settings = state
            .project_settings
            .lock()
            .map_err(|_| "failed to lock project settings".to_string())?;
        settings.global_shortcuts_enabled = enabled;
    }

    state.mark_dirty()?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_global_shortcuts_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| "failed to lock shortcut manager".to_string())?;

    Ok(manager.enabled())
}
