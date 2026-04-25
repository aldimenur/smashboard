use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEvent {
    pub event_id: String,
    pub time_ms: f64,
    pub slot_id: String,
    pub audio_path: String,
    pub label: String,
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
}
