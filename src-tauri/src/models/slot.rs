use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::audio::decoder::decode_audio;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub id: String,
    #[serde(default)]
    pub position: usize,
    pub label: String,
    pub audio_path: String,
    #[serde(default)]
    pub image_data_url: Option<String>,
    #[serde(default)]
    pub icon_name: Option<String>,
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
    pub color: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}

impl Slot {
    pub fn new(audio_path: String, label: Option<String>, position: usize) -> Result<Self, String> {
        let path_buf = PathBuf::from(&audio_path);
        let path = path_buf.as_path();
        if !path.exists() {
            return Err(format!("audio file not found: {}", path.display()));
        }

        let decoded = decode_audio(path)?;
        let fallback_label = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Untitled");

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            position,
            label: label.unwrap_or_else(|| fallback_label.to_string()),
            audio_path,
            image_data_url: None,
            icon_name: None,
            shortcut: String::new(),
            gain: 1.0,
            duration_ms: decoded.duration_ms,
            color: color_from_path(path),
            created_at: Utc::now(),
        })
    }
}

fn color_from_path(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    let hue = hash % 360;
    let saturation = 55 + (hash / 360 % 25);
    let lightness = 45 + (hash / 360 / 25 % 20);

    format!("hsl({hue}, {saturation}%, {lightness}%)")
}
