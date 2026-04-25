use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

pub struct PlaybackEngine {
    status: PlaybackStatus,
    current_time_ms: f64,
    start_instant: Option<Instant>,
}

impl PlaybackEngine {
    pub fn new() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            current_time_ms: 0.0,
            start_instant: None,
        }
    }

    pub fn play(&mut self, from_time_ms: f64) {
        self.current_time_ms = from_time_ms.max(0.0);
        self.start_instant = Some(Instant::now());
        self.status = PlaybackStatus::Playing;
    }

    pub fn pause(&mut self) {
        if self.status == PlaybackStatus::Playing {
            self.current_time_ms = self.get_current_time();
            self.start_instant = None;
            self.status = PlaybackStatus::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.current_time_ms = self.get_current_time();
        self.start_instant = None;
        self.status = PlaybackStatus::Stopped;
    }

    pub fn seek(&mut self, time_ms: f64) {
        self.current_time_ms = time_ms.max(0.0);
        if self.status == PlaybackStatus::Playing {
            self.start_instant = Some(Instant::now());
        }
    }

    pub fn get_current_time(&self) -> f64 {
        if self.status == PlaybackStatus::Playing {
            let elapsed_ms = self
                .start_instant
                .map(|instant| instant.elapsed().as_secs_f64() * 1000.0)
                .unwrap_or(0.0);
            self.current_time_ms + elapsed_ms
        } else {
            self.current_time_ms
        }
    }

    pub fn status(&self) -> PlaybackStatus {
        self.status.clone()
    }
}
