use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::slot::Slot;
use super::timeline::TimelineEvent;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub version: String,
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub settings: ProjectSettings,
    pub slots: Vec<Slot>,
    pub timeline: TimelineData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ProjectSettings {
    pub global_shortcuts_enabled: bool,
    pub audio_buffer_size: u32,
    pub frame_rate: u32,
    pub board_rows: u8,
    pub board_columns: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimelineData {
    pub events: Vec<TimelineEvent>,
    pub total_duration_ms: f64,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            global_shortcuts_enabled: false,
            audio_buffer_size: 512,
            frame_rate: 30,
            board_rows: 5,
            board_columns: 5,
        }
    }
}

impl Project {
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create project directory: {err}"))?;
        }

        let json =
            serde_json::to_string_pretty(self).map_err(|err| format!("failed to serialize project: {err}"))?;

        std::fs::write(path, json).map_err(|err| format!("failed to write project file: {err}"))?;

        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|err| format!("failed to read project file {}: {err}", path.display()))?;

        serde_json::from_str(&json)
            .map_err(|err| format!("failed to parse project file {}: {err}", path.display()))
    }

    pub fn validate_audio_paths(&self) -> Vec<String> {
        let mut missing = Vec::new();

        for slot in &self.slots {
            if !Path::new(&slot.audio_path).exists() {
                missing.push(slot.audio_path.clone());
            }
        }

        missing.sort();
        missing.dedup();
        missing
    }
}
