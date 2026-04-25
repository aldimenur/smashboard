use tauri::{AppHandle, State};

use crate::remote::RemoteControlStatus;
use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn get_remote_control_status(state: State<'_, AppState>) -> Result<RemoteControlStatus, String> {
    let manager = state
        .remote_control
        .lock()
        .map_err(|_| "failed to lock remote control manager".to_string())?;
    Ok(manager.status())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn start_remote_control(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    port: Option<u16>,
) -> Result<RemoteControlStatus, String> {
    let mut manager = state
        .remote_control
        .lock()
        .map_err(|_| "failed to lock remote control manager".to_string())?;
    manager.start(app_handle, &state, port.unwrap_or(8765))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn stop_remote_control(state: State<'_, AppState>) -> Result<RemoteControlStatus, String> {
    let mut manager = state
        .remote_control
        .lock()
        .map_err(|_| "failed to lock remote control manager".to_string())?;
    Ok(manager.stop())
}
