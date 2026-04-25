use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use hound::SampleFormat;
use minimp3::{Decoder, Error as Mp3Error};

#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: f64,
}

pub fn decode_audio(path: &Path) -> Result<AudioBuffer, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| format!("unsupported file type: {}", path.display()))?;

    match extension.as_str() {
        "wav" => decode_wav(path),
        "mp3" => decode_mp3(path),
        _ => Err(format!("unsupported audio format: .{}", extension)),
    }
}

pub fn decode_wav(path: &Path) -> Result<AudioBuffer, String> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|err| format!("failed to open WAV file {}: {err}", path.display()))?;

    let spec = reader.spec();
    let channels = spec.channels;
    let sample_rate = spec.sample_rate;

    let samples = match spec.sample_format {
        SampleFormat::Int => {
            let bits = spec.bits_per_sample.max(1);
            let max_value = ((1_i64 << (bits.saturating_sub(1) as u32)) - 1) as f32;
            if max_value <= 0.0 {
                return Err(format!("invalid WAV bit depth for {}", path.display()));
            }

            reader
                .samples::<i32>()
                .map(|sample| {
                    sample
                        .map(|value| (value as f32 / max_value).clamp(-1.0, 1.0))
                        .map_err(|err| {
                            format!("failed to decode WAV sample in {}: {err}", path.display())
                        })
                })
                .collect::<Result<Vec<f32>, String>>()?
        }
        SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| {
                sample.map(|value| value.clamp(-1.0, 1.0)).map_err(|err| {
                    format!("failed to decode WAV sample in {}: {err}", path.display())
                })
            })
            .collect::<Result<Vec<f32>, String>>()?,
    };

    finalize_buffer(samples, sample_rate, channels)
}

pub fn decode_mp3(path: &Path) -> Result<AudioBuffer, String> {
    let file = File::open(path)
        .map_err(|err| format!("failed to open MP3 file {}: {err}", path.display()))?;

    let mut decoder = Decoder::new(BufReader::new(file));
    let mut samples = Vec::new();
    let mut sample_rate: Option<u32> = None;
    let mut channels: Option<u16> = None;

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                let frame_sample_rate = u32::try_from(frame.sample_rate)
                    .map_err(|_| format!("invalid MP3 sample rate in {}", path.display()))?;
                let frame_channels = u16::try_from(frame.channels)
                    .map_err(|_| format!("invalid MP3 channel count in {}", path.display()))?;

                if let Some(value) = sample_rate {
                    if value != frame_sample_rate {
                        return Err(format!(
                            "unsupported variable sample rate MP3 in {}",
                            path.display()
                        ));
                    }
                } else {
                    sample_rate = Some(frame_sample_rate);
                }

                if let Some(value) = channels {
                    if value != frame_channels {
                        return Err(format!(
                            "unsupported variable channel count MP3 in {}",
                            path.display()
                        ));
                    }
                } else {
                    channels = Some(frame_channels);
                }

                samples.extend(
                    frame
                        .data
                        .into_iter()
                        .map(|value| (value as f32 / i16::MAX as f32).clamp(-1.0, 1.0)),
                );
            }
            Err(Mp3Error::Eof) => break,
            Err(err) => {
                return Err(format!(
                    "failed to decode MP3 frame in {}: {err}",
                    path.display()
                ));
            }
        }
    }

    let sample_rate =
        sample_rate.ok_or_else(|| format!("no audio frames found in {}", path.display()))?;
    let channels =
        channels.ok_or_else(|| format!("no audio channels found in {}", path.display()))?;

    finalize_buffer(samples, sample_rate, channels)
}

fn finalize_buffer(
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<AudioBuffer, String> {
    if channels == 0 {
        return Err("audio channels cannot be zero".to_string());
    }

    if sample_rate == 0 {
        return Err("sample rate cannot be zero".to_string());
    }

    if samples.is_empty() {
        return Err("audio file produced no samples".to_string());
    }

    let frames = samples.len() as f64 / channels as f64;
    let duration_ms = (frames / sample_rate as f64) * 1000.0;

    Ok(AudioBuffer {
        samples,
        sample_rate,
        channels,
        duration_ms,
    })
}
