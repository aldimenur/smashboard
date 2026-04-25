use crate::models::timeline::TimelineEvent;

#[derive(Default)]
pub struct TimelineState {
    pub events: Vec<TimelineEvent>,
    pub total_duration_ms: f64,
    pub playhead_position_ms: f64,
}

impl TimelineState {
    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
        self.events
            .sort_by(|left, right| left.time_ms.total_cmp(&right.time_ms));
        self.recalculate_duration();
    }

    pub fn add_events(&mut self, events: Vec<TimelineEvent>) {
        self.events.extend(events);
        self.events
            .sort_by(|left, right| left.time_ms.total_cmp(&right.time_ms));
        self.recalculate_duration();
    }

    pub fn delete_events(&mut self, event_ids: &[String]) {
        self.events.retain(|event| !event_ids.contains(&event.event_id));
        self.recalculate_duration();
    }

    pub fn update_event_time(&mut self, event_id: &str, new_time_ms: f64) {
        if let Some(event) = self.events.iter_mut().find(|event| event.event_id == event_id) {
            event.time_ms = new_time_ms.max(0.0);
        }

        self.events
            .sort_by(|left, right| left.time_ms.total_cmp(&right.time_ms));
        self.recalculate_duration();
    }

    pub fn set_playhead_position(&mut self, time_ms: f64) {
        self.playhead_position_ms = time_ms.max(0.0);
    }

    pub fn get_events_at_time(&self, time_ms: f64, lookahead_ms: f64) -> Vec<TimelineEvent> {
        self.events
            .iter()
            .filter(|event| event.time_ms >= time_ms && event.time_ms < time_ms + lookahead_ms)
            .cloned()
            .collect()
    }

    pub fn get_events_before_time(&self, time_ms: f64) -> Vec<String> {
        self.events
            .iter()
            .filter(|event| event.time_ms < time_ms)
            .map(|event| event.event_id.clone())
            .collect()
    }

    pub fn recalculate_duration(&mut self) {
        self.total_duration_ms = self
            .events
            .iter()
            .map(|event| event.time_ms + event.duration_ms)
            .fold(0.0, f64::max);
    }
}
