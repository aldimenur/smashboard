use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct AutosaveManager {
    last_save_time: Instant,
    interval_secs: u64,
}

impl AutosaveManager {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            last_save_time: Instant::now(),
            interval_secs,
        }
    }

    pub fn should_autosave(&self) -> bool {
        self.last_save_time.elapsed().as_secs() >= self.interval_secs
    }

    pub fn mark_saved(&mut self) {
        self.last_save_time = Instant::now();
    }

    pub fn get_autosave_path() -> PathBuf {
        if let Ok(app_data) = std::env::var("APPDATA") {
            return Path::new(&app_data)
                .join("SFXBoard")
                .join("autosave.sfxproj");
        }

        Path::new(".").join("autosave.sfxproj")
    }
}
