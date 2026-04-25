use std::time::{Duration, Instant};

use chrono::Utc;
use uuid::Uuid;

use crate::models::recording::{RecordingSession, RecordingStatus};
use crate::models::slot::Slot;
use crate::models::timeline::TimelineEvent;

pub struct RecordingEngine {
    session: Option<RecordingSession>,
    start_time: Option<Instant>,
    paused_duration: Duration,
    pause_started_at: Option<Instant>,
}

impl RecordingEngine {
    pub fn new() -> Self {
        Self {
            session: None,
            start_time: None,
            paused_duration: Duration::ZERO,
            pause_started_at: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if matches!(
            self.status(),
            RecordingStatus::Recording | RecordingStatus::Paused
        ) {
            return Err("recording session already active".to_string());
        }

        self.session = Some(RecordingSession {
            session_id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            status: RecordingStatus::Recording,
            current_time_ms: 0.0,
            events_buffer: Vec::new(),
        });
        self.start_time = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if self.status() != RecordingStatus::Recording {
            return Err("recording is not running".to_string());
        }

        self.pause_started_at = Some(Instant::now());
        let current_time_ms = self.get_current_time_ms();

        if let Some(session) = self.session.as_mut() {
            session.status = RecordingStatus::Paused;
            session.current_time_ms = current_time_ms;
        }

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        if self.status() != RecordingStatus::Paused {
            return Err("recording is not paused".to_string());
        }

        if let Some(pause_started_at) = self.pause_started_at.take() {
            self.paused_duration += pause_started_at.elapsed();
        }

        if let Some(session) = self.session.as_mut() {
            session.status = RecordingStatus::Recording;
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<TimelineEvent>, String> {
        let status = self.status();
        if matches!(status, RecordingStatus::Idle | RecordingStatus::Stopped) {
            return Err("no active recording session".to_string());
        }

        let current_time_ms = self.get_current_time_ms();
        let mut session = self
            .session
            .take()
            .ok_or_else(|| "recording session not found".to_string())?;

        session.status = RecordingStatus::Stopped;
        session.current_time_ms = current_time_ms;

        let events = session.events_buffer.clone();
        self.start_time = None;
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;

        Ok(events)
    }

    pub fn capture_event(&mut self, slot: &Slot) -> Result<TimelineEvent, String> {
        if self.status() != RecordingStatus::Recording {
            return Err("recording is not active".to_string());
        }

        let time_ms = self.get_current_time_ms();
        let event = TimelineEvent {
            event_id: Uuid::new_v4().to_string(),
            time_ms,
            slot_id: slot.id.clone(),
            audio_path: slot.audio_path.clone(),
            label: slot.label.clone(),
            shortcut: slot.shortcut.clone(),
            gain: slot.gain,
            duration_ms: slot.duration_ms,
        };

        if let Some(session) = self.session.as_mut() {
            session.events_buffer.push(event.clone());
            session.current_time_ms = time_ms;
            Ok(event)
        } else {
            Err("recording session not found".to_string())
        }
    }

    pub fn get_current_time_ms(&self) -> f64 {
        match self.status() {
            RecordingStatus::Recording => {
                let start = self.start_time.unwrap_or_else(Instant::now);
                let elapsed = start.elapsed().saturating_sub(self.paused_duration);
                elapsed.as_secs_f64() * 1000.0
            }
            RecordingStatus::Paused => {
                let start = self.start_time.unwrap_or_else(Instant::now);
                let pause_started = self.pause_started_at.unwrap_or_else(Instant::now);
                let elapsed_until_pause = pause_started
                    .saturating_duration_since(start)
                    .saturating_sub(self.paused_duration);
                elapsed_until_pause.as_secs_f64() * 1000.0
            }
            _ => self
                .session
                .as_ref()
                .map(|session| session.current_time_ms)
                .unwrap_or(0.0),
        }
    }

    pub fn status(&self) -> RecordingStatus {
        self.session
            .as_ref()
            .map(|session| session.status.clone())
            .unwrap_or(RecordingStatus::Idle)
    }

    pub fn is_recording(&self) -> bool {
        self.status() == RecordingStatus::Recording
    }
}
