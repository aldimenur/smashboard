use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink, Source};

use super::decoder::AudioBuffer;
use super::mixer::normalize_gain;

#[derive(Clone, Debug)]
pub struct PlaybackHandle {
    pub id: u64,
    pub duration_ms: f64,
}

struct PlayCommand {
    playback_id: u64,
    buffer: AudioBuffer,
    gain: f32,
    response_tx: mpsc::Sender<Result<PlaybackHandle, String>>,
}

pub struct AudioEngine {
    command_tx: mpsc::Sender<PlayCommand>,
    next_playback_id: AtomicU64,
    active_playbacks: Arc<Mutex<usize>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let (command_tx, command_rx) = mpsc::channel::<PlayCommand>();
        let (init_tx, init_rx) = mpsc::channel::<Result<(), String>>();
        let active_playbacks = Arc::new(Mutex::new(0usize));
        let active_playbacks_thread = Arc::clone(&active_playbacks);

        std::thread::Builder::new()
            .name("audio-engine".to_string())
            .spawn(move || {
                let (stream, handle) = match OutputStream::try_default() {
                    Ok(value) => {
                        let _ = init_tx.send(Ok(()));
                        value
                    }
                    Err(err) => {
                        let _ =
                            init_tx.send(Err(format!("failed to initialize audio output: {err}")));
                        return;
                    }
                };

                let _stream = stream;
                let mut active_sinks: HashMap<u64, Arc<Sink>> = HashMap::new();

                while let Ok(command) = command_rx.recv() {
                    active_sinks.retain(|_, sink| !sink.empty());

                    let concurrent = active_sinks.len() + 1;
                    let normalized_gain = normalize_gain(command.gain, concurrent);

                    let result = (|| {
                        let sink = Arc::new(
                            Sink::try_new(&handle)
                                .map_err(|err| format!("failed to create sink: {err}"))?,
                        );
                        let source = SamplesBuffer::new(
                            command.buffer.channels,
                            command.buffer.sample_rate,
                            command.buffer.samples,
                        )
                        .amplify(normalized_gain);

                        sink.append(source);
                        sink.play();

                        active_sinks.insert(command.playback_id, Arc::clone(&sink));

                        if let Ok(mut guard) = active_playbacks_thread.lock() {
                            *guard = active_sinks.len();
                        }

                        Ok(PlaybackHandle {
                            id: command.playback_id,
                            duration_ms: command.buffer.duration_ms,
                        })
                    })();

                    let _ = command.response_tx.send(result);
                }
            })
            .map_err(|err| format!("failed to start audio thread: {err}"))?;

        init_rx
            .recv()
            .map_err(|_| "audio thread failed to initialize".to_string())??;

        Ok(Self {
            command_tx,
            next_playback_id: AtomicU64::new(1),
            active_playbacks,
        })
    }

    pub fn play(&self, buffer: AudioBuffer, gain: f32) -> Result<PlaybackHandle, String> {
        let playback_id = self.next_playback_id.fetch_add(1, Ordering::Relaxed);
        let (response_tx, response_rx) = mpsc::channel::<Result<PlaybackHandle, String>>();

        self.command_tx
            .send(PlayCommand {
                playback_id,
                buffer,
                gain,
                response_tx,
            })
            .map_err(|_| "audio engine is unavailable".to_string())?;

        response_rx
            .recv()
            .map_err(|_| "audio engine did not return a response".to_string())?
    }

    pub fn active_playback_count(&self) -> usize {
        self.active_playbacks
            .lock()
            .map(|value| *value)
            .unwrap_or(0)
    }
}
