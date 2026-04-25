use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use chrono::Utc;
use hound::{SampleFormat, WavSpec, WavWriter};
use serde::Serialize;

use crate::audio::decoder::decode_audio;
use crate::models::project::Project;
use crate::models::slot::Slot;
use crate::models::timeline::TimelineEvent;

const EXPORT_SAMPLE_RATE: u32 = 44_100;

struct StereoMixBuffer {
    left: Vec<f32>,
    right: Vec<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineExport {
    pub export_version: String,
    pub exported_at: chrono::DateTime<Utc>,
    pub project_name: String,
    pub frame_rate: u32,
    pub timeline: TimelineExportData,
    pub slots: Vec<SlotExportData>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineExportData {
    pub total_duration_ms: f64,
    pub total_duration_frames: u32,
    pub event_count: usize,
    pub events: Vec<EventExportData>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventExportData {
    pub event_id: String,
    pub time_ms: f64,
    pub time_frames: u32,
    pub time_formatted: String,
    pub label: String,
    pub audio_file: String,
    pub audio_path: String,
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
    pub duration_frames: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotExportData {
    pub id: String,
    pub label: String,
    pub audio_file: String,
    pub audio_path: String,
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
    pub usage_count: usize,
}

pub fn export_timeline_to_wav(
    events: &[TimelineEvent],
    output_path: &Path,
    allow_missing_files: bool,
) -> Result<(), String> {
    let mix = render_mix(events, allow_missing_files)?;
    write_wav_buffer(&mix, output_path)
}

pub fn export_timeline_to_mp3(
    events: &[TimelineEvent],
    output_path: &Path,
    allow_missing_files: bool,
) -> Result<(), String> {
    let temp_wav = std::env::temp_dir().join(format!(
        "sfx-board-export-{}.wav",
        uuid::Uuid::new_v4().simple()
    ));

    export_timeline_to_wav(events, &temp_wav, allow_missing_files)?;

    let ffmpeg_attempt = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(&temp_wav)
        .arg("-codec:a")
        .arg("libmp3lame")
        .arg("-b:a")
        .arg("320k")
        .arg(output_path)
        .output();

    let success = match ffmpeg_attempt {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };

    if !success {
        let lame_attempt = Command::new("lame")
            .arg("-b")
            .arg("320")
            .arg(&temp_wav)
            .arg(output_path)
            .output();

        if !matches!(lame_attempt, Ok(output) if output.status.success()) {
            let _ = std::fs::remove_file(&temp_wav);
            return Err(
                "failed to export MP3: encoder unavailable (install ffmpeg or lame)".to_string(),
            );
        }
    }

    let _ = std::fs::remove_file(temp_wav);

    Ok(())
}

pub fn export_timeline_to_json(project: &Project, output_path: &Path) -> Result<(), String> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create export directory: {err}"))?;
    }

    let frame_rate = project.settings.frame_rate;
    let events = project
        .timeline
        .events
        .iter()
        .map(|event| EventExportData {
            event_id: event.event_id.clone(),
            time_ms: event.time_ms,
            time_frames: ms_to_frames(event.time_ms, frame_rate),
            time_formatted: format_time(event.time_ms),
            label: event.label.clone(),
            audio_file: Path::new(&event.audio_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_string(),
            audio_path: event.audio_path.clone(),
            shortcut: event.shortcut.clone(),
            gain: event.gain,
            duration_ms: event.duration_ms,
            duration_frames: ms_to_frames(event.duration_ms, frame_rate),
        })
        .collect::<Vec<_>>();

    let usage_by_slot = project
        .timeline
        .events
        .iter()
        .fold(HashMap::<&str, usize>::new(), |mut map, event| {
            let counter = map.entry(event.slot_id.as_str()).or_insert(0);
            *counter += 1;
            map
        });

    let slots = project
        .slots
        .iter()
        .map(|slot| SlotExportData {
            id: slot.id.clone(),
            label: slot.label.clone(),
            audio_file: Path::new(&slot.audio_path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_string(),
            audio_path: slot.audio_path.clone(),
            shortcut: slot.shortcut.clone(),
            gain: slot.gain,
            duration_ms: slot.duration_ms,
            usage_count: *usage_by_slot.get(slot.id.as_str()).unwrap_or(&0),
        })
        .collect::<Vec<_>>();

    let export = TimelineExport {
        export_version: "1.0.0".to_string(),
        exported_at: Utc::now(),
        project_name: project.project_name.clone(),
        frame_rate,
        timeline: TimelineExportData {
            total_duration_ms: project.timeline.total_duration_ms,
            total_duration_frames: ms_to_frames(project.timeline.total_duration_ms, frame_rate),
            event_count: project.timeline.events.len(),
            events,
        },
        slots,
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|err| format!("failed to serialize export json: {err}"))?;

    std::fs::write(output_path, json).map_err(|err| format!("failed to write JSON export: {err}"))
}

fn render_mix(events: &[TimelineEvent], allow_missing_files: bool) -> Result<StereoMixBuffer, String> {
    let max_end_time_ms = events
        .iter()
        .map(|event| event.time_ms + event.duration_ms)
        .fold(0.0, f64::max)
        + 1_000.0;

    let frame_count = ((max_end_time_ms / 1000.0) * EXPORT_SAMPLE_RATE as f64).ceil() as usize;
    let mut left = vec![0.0f32; frame_count.max(1)];
    let mut right = vec![0.0f32; frame_count.max(1)];

    for event in events {
        let decoded = match decode_audio(Path::new(&event.audio_path)) {
            Ok(decoded) => decoded,
            Err(err) if allow_missing_files => {
                tracing::warn!(
                    event_id = event.event_id,
                    audio_path = event.audio_path,
                    ?err,
                    "skipping event during export due to missing or unreadable audio",
                );
                continue;
            }
            Err(err) => return Err(err),
        };
        let stereo_frames = to_stereo_frames(
            &decoded.samples,
            decoded.channels as usize,
            decoded.sample_rate,
            EXPORT_SAMPLE_RATE,
        );

        let start_index = ((event.time_ms / 1000.0) * EXPORT_SAMPLE_RATE as f64).round() as usize;

        for (frame_index, (l, r)) in stereo_frames.iter().copied().enumerate() {
            let target_index = start_index + frame_index;
            if target_index >= left.len() {
                break;
            }

            left[target_index] += l * event.gain;
            right[target_index] += r * event.gain;
        }
    }

    normalize(&mut left, &mut right);

    Ok(StereoMixBuffer { left, right })
}

fn normalize(left: &mut [f32], right: &mut [f32]) {
    let peak = left
        .iter()
        .chain(right.iter())
        .map(|sample| sample.abs())
        .fold(0.0f32, f32::max);

    if peak <= 1.0 {
        return;
    }

    let factor = 1.0 / peak;
    for sample in left.iter_mut() {
        *sample *= factor;
    }
    for sample in right.iter_mut() {
        *sample *= factor;
    }
}

fn write_wav_buffer(buffer: &StereoMixBuffer, output_path: &Path) -> Result<(), String> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create export directory: {err}"))?;
    }

    let spec = WavSpec {
        channels: 2,
        sample_rate: EXPORT_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output_path, spec)
        .map_err(|err| format!("failed to create WAV writer: {err}"))?;

    for index in 0..buffer.left.len() {
        let left = (buffer.left[index].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        let right = (buffer.right[index].clamp(-1.0, 1.0) * i16::MAX as f32) as i16;

        writer
            .write_sample(left)
            .map_err(|err| format!("failed writing WAV sample: {err}"))?;
        writer
            .write_sample(right)
            .map_err(|err| format!("failed writing WAV sample: {err}"))?;
    }

    writer
        .finalize()
        .map_err(|err| format!("failed finalizing WAV file: {err}"))?;

    Ok(())
}

fn to_stereo_frames(
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
    target_sample_rate: u32,
) -> Vec<(f32, f32)> {
    let channels = channels.max(1);
    let input_frames = samples.len() / channels;

    let mut stereo = Vec::with_capacity(input_frames);
    for frame_index in 0..input_frames {
        let base = frame_index * channels;
        let left = samples[base];
        let right = if channels > 1 { samples[base + 1] } else { left };
        stereo.push((left, right));
    }

    if sample_rate == target_sample_rate || stereo.is_empty() {
        return stereo;
    }

    let ratio = sample_rate as f64 / target_sample_rate as f64;
    let output_frames = ((stereo.len() as f64) / ratio).ceil() as usize;

    (0..output_frames)
        .map(|index| {
            let position = index as f64 * ratio;
            let lower = position.floor() as usize;
            let upper = (lower + 1).min(stereo.len().saturating_sub(1));
            let alpha = (position - lower as f64) as f32;

            let (l0, r0) = stereo[lower];
            let (l1, r1) = stereo[upper];

            (l0 + (l1 - l0) * alpha, r0 + (r1 - r0) * alpha)
        })
        .collect()
}

fn ms_to_frames(ms: f64, frame_rate: u32) -> u32 {
    ((ms / 1000.0) * frame_rate as f64).round() as u32
}

fn format_time(ms: f64) -> String {
    let total_ms = ms.max(0.0).round() as u64;
    let minutes = total_ms / 60_000;
    let seconds = (total_ms % 60_000) / 1_000;
    let millis = total_ms % 1_000;

    format!("{minutes:02}:{seconds:02}.{millis:03}")
}

pub fn project_from_state(
    project_name: &str,
    created_at: chrono::DateTime<Utc>,
    modified_at: chrono::DateTime<Utc>,
    settings: &crate::models::project::ProjectSettings,
    slots: &[Slot],
    events: &[TimelineEvent],
    total_duration_ms: f64,
) -> Project {
    Project {
        version: "1.0.0".to_string(),
        project_name: project_name.to_string(),
        created_at,
        modified_at,
        settings: settings.clone(),
        slots: slots.to_vec(),
        timeline: crate::models::project::TimelineData {
            events: events.to_vec(),
            total_duration_ms,
        },
    }
}
