use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::timeline::TimelineEvent;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingStatus {
    Idle,
    Recording,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingSession {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub status: RecordingStatus,
    pub current_time_ms: f64,
    pub events_buffer: Vec<TimelineEvent>,
}
