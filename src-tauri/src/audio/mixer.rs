pub fn normalize_gain(requested_gain: f32, concurrent_instances: usize) -> f32 {
    let clamped = requested_gain.clamp(0.0, 2.0);
    if concurrent_instances <= 1 {
        return clamped;
    }

    let scale = (concurrent_instances as f32).sqrt();
    (clamped / scale).clamp(0.0, 1.0)
}
