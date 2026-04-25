pub mod audio;
pub mod commands;
pub mod models;

use std::sync::{Arc, Mutex};

use audio::engine::AudioEngine;
use models::slot::Slot;

pub struct AppState {
    pub slots: Arc<Mutex<Vec<Slot>>>,
    pub audio_engine: Arc<AudioEngine>,
    pub max_slots: usize,
}

impl AppState {
    fn new() -> Result<Self, String> {
        let audio_engine = AudioEngine::new()?;
        Ok(Self {
            slots: Arc::new(Mutex::new(Vec::new())),
            audio_engine: Arc::new(audio_engine),
            max_slots: 64,
        })
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
        .invoke_handler(tauri::generate_handler![
            commands::slot_commands::add_slot,
            commands::slot_commands::update_slot,
            commands::slot_commands::delete_slot,
            commands::slot_commands::get_all_slots,
            commands::slot_commands::trigger_slot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
