use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;

use crate::commands::slot_commands::trigger_slot_with_shared;
use crate::models::slot::Slot;
use crate::models::timeline::TimelineEvent;
use crate::timeline::playback::PlaybackStatus;
use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteControlStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub token: Option<String>,
    pub url: Option<String>,
}

#[derive(Default)]
pub struct RemoteControlManager {
    running: Arc<AtomicBool>,
    port: Option<u16>,
    token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoteStatePayload {
    project_name: String,
    board_rows: u8,
    board_columns: u8,
    playhead_ms: f64,
    playback_status: PlaybackStatus,
    recording_status: crate::models::recording::RecordingStatus,
    slots: Vec<Slot>,
    timeline_events: Vec<TimelineEvent>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteCommandPayload {
    kind: String,
    slot_id: Option<String>,
}

impl RemoteControlManager {
    pub fn status(&self) -> RemoteControlStatus {
        let running = self.running.load(Ordering::SeqCst);
        let url = self.port.and_then(|port| {
            self.token
                .as_ref()
                .map(|token| format!("http://{}:{}/?token={token}", local_ip_guess(), port))
        });

        RemoteControlStatus {
            running,
            port: self.port,
            token: self.token.clone(),
            url,
        }
    }

    pub fn start(
        &mut self,
        app_handle: AppHandle,
        state: &AppState,
        port: u16,
    ) -> Result<RemoteControlStatus, String> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(self.status());
        }

        let listener = TcpListener::bind(("0.0.0.0", port))
            .map_err(|err| format!("failed to start remote server on port {port}: {err}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|err| format!("failed to configure remote server socket: {err}"))?;

        let running = Arc::new(AtomicBool::new(true));
        let running_for_thread = running.clone();
        let token = Uuid::new_v4().to_string();

        let slots = state.slots.clone();
        let audio_engine = state.audio_engine.clone();
        let recording_engine = state.recording_engine.clone();
        let timeline_state = state.timeline_state.clone();
        let playback_engine = state.playback_engine.clone();
        let project_settings = state.project_settings.clone();
        let project_name = state.project_name.clone();
        let token_for_thread = token.clone();
        let app_handle_for_thread = app_handle.clone();

        thread::Builder::new()
            .name("remote-control-server".to_string())
            .spawn(move || {
                while running_for_thread.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let token = token_for_thread.clone();
                            let running = running_for_thread.clone();
                            let slots = slots.clone();
                            let audio_engine = audio_engine.clone();
                            let recording_engine = recording_engine.clone();
                            let timeline_state = timeline_state.clone();
                            let playback_engine = playback_engine.clone();
                            let project_settings = project_settings.clone();
                            let project_name = project_name.clone();
                            let app_handle = app_handle_for_thread.clone();
                            thread::spawn(move || {
                                let _ = handle_connection(
                                    stream,
                                    token,
                                    running,
                                    slots,
                                    audio_engine,
                                    recording_engine,
                                    timeline_state,
                                    playback_engine,
                                    project_settings,
                                    project_name,
                                    app_handle,
                                );
                            });
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(20));
                        }
                        Err(_) => {
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            })
            .map_err(|err| format!("failed to start remote server thread: {err}"))?;

        self.running = running;
        self.port = Some(port);
        self.token = Some(token);

        Ok(self.status())
    }

    pub fn stop(&mut self) -> RemoteControlStatus {
        self.running.store(false, Ordering::SeqCst);
        self.running = Arc::new(AtomicBool::new(false));
        self.port = None;
        self.token = None;
        self.status()
    }
}

fn handle_connection(
    mut stream: TcpStream,
    token: String,
    running: Arc<AtomicBool>,
    slots: Arc<Mutex<Vec<Slot>>>,
    audio_engine: Arc<crate::audio::engine::AudioEngine>,
    recording_engine: Arc<Mutex<crate::recording::engine::RecordingEngine>>,
    timeline_state: Arc<Mutex<crate::timeline::state::TimelineState>>,
    playback_engine: Arc<Mutex<crate::timeline::playback::PlaybackEngine>>,
    project_settings: Arc<Mutex<crate::models::project::ProjectSettings>>,
    project_name: Arc<Mutex<String>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let peer_ip = stream
        .peer_addr()
        .ok()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));
    if !is_lan_or_loopback(peer_ip) {
        return write_http_response(
            &mut stream,
            "403 Forbidden",
            "application/json",
            br#"{"error":"lan_only"}"#,
        );
    }

    let request = read_http_request(&mut stream)?;
    let token_ok = request
        .query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .any(|(key, value)| key == "token" && value == token);

    if request.path == "/" {
        return write_http_response(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            remote_html().as_bytes(),
        );
    }

    if request.method == "GET" && request.path == "/favicon.ico" {
        return write_http_response(&mut stream, "204 No Content", "image/x-icon", &[]);
    }

    if request.method == "GET" && request.path == "/remote/styles.css" {
        return write_http_response(
            &mut stream,
            "200 OK",
            "text/css; charset=utf-8",
            remote_css().as_bytes(),
        );
    }

    if request.method == "GET" && request.path == "/remote/app.js" {
        return write_http_response(
            &mut stream,
            "200 OK",
            "application/javascript; charset=utf-8",
            remote_app_js().as_bytes(),
        );
    }

    if !token_ok {
        return write_http_response(
            &mut stream,
            "401 Unauthorized",
            "application/json",
            br#"{"error":"unauthorized"}"#,
        );
    }

    if request.method == "GET" && request.path == "/api/state" {
        let payload = snapshot_state(
            &slots,
            &timeline_state,
            &playback_engine,
            &recording_engine,
            &project_settings,
            &project_name,
        )?;
        let json = serde_json::to_vec(&payload).map_err(|err| format!("json encode failed: {err}"))?;
        return write_http_response(&mut stream, "200 OK", "application/json", &json);
    }

    if request.method == "GET" && request.path == "/api/events" {
        return write_state_event_stream(
            &mut stream,
            &running,
            &slots,
            &timeline_state,
            &playback_engine,
            &recording_engine,
            &project_settings,
            &project_name,
        );
    }

    if request.method == "POST" && request.path == "/api/command" {
        let command: RemoteCommandPayload =
            serde_json::from_slice(&request.body).map_err(|err| format!("invalid command payload: {err}"))?;
        execute_remote_command(
            &command,
            &slots,
            &audio_engine,
            &recording_engine,
            &app_handle,
        )?;
        return write_http_response(&mut stream, "200 OK", "application/json", br#"{"ok":true}"#);
    }

    write_http_response(
        &mut stream,
        "404 Not Found",
        "application/json",
        br#"{"error":"not_found"}"#,
    )
}

fn write_state_event_stream(
    stream: &mut TcpStream,
    running: &Arc<AtomicBool>,
    slots: &Arc<Mutex<Vec<Slot>>>,
    timeline_state: &Arc<Mutex<crate::timeline::state::TimelineState>>,
    playback_engine: &Arc<Mutex<crate::timeline::playback::PlaybackEngine>>,
    recording_engine: &Arc<Mutex<crate::recording::engine::RecordingEngine>>,
    project_settings: &Arc<Mutex<crate::models::project::ProjectSettings>>,
    project_name: &Arc<Mutex<String>>,
) -> Result<(), String> {
    let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nX-Accel-Buffering: no\r\n\r\n";
    stream
        .write_all(headers.as_bytes())
        .map_err(|err| format!("write event-stream headers failed: {err}"))?;

    let mut last_json = String::new();
    let mut keepalive_ticks = 0u8;
    while running.load(Ordering::SeqCst) {
        let payload = snapshot_state(
            slots,
            timeline_state,
            playback_engine,
            recording_engine,
            project_settings,
            project_name,
        )?;
        let json =
            serde_json::to_string(&payload).map_err(|err| format!("event-stream json encode failed: {err}"))?;

        if json != last_json {
            let frame = format!("event: state\ndata: {json}\n\n");
            if !try_write_stream(stream, frame.as_bytes()) {
                return Ok(());
            }
            last_json = json;
            keepalive_ticks = 0;
        } else {
            keepalive_ticks = keepalive_ticks.saturating_add(1);
            if keepalive_ticks >= 20 {
                if !try_write_stream(stream, b": keepalive\n\n") {
                    return Ok(());
                }
                keepalive_ticks = 0;
            }
        }

        thread::sleep(Duration::from_millis(120));
    }

    Ok(())
}

fn try_write_stream(stream: &mut TcpStream, bytes: &[u8]) -> bool {
    match stream.write_all(bytes).and_then(|_| stream.flush()) {
        Ok(_) => true,
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
            ) =>
        {
            false
        }
        Err(_) => false,
    }
}

fn execute_remote_command(
    command: &RemoteCommandPayload,
    slots: &Arc<Mutex<Vec<Slot>>>,
    audio_engine: &Arc<crate::audio::engine::AudioEngine>,
    recording_engine: &Arc<Mutex<crate::recording::engine::RecordingEngine>>,
    app_handle: &AppHandle,
) -> Result<(), String> {
    match command.kind.as_str() {
        "trigger_slot" => {
            let slot_id = command
                .slot_id
                .clone()
                .ok_or_else(|| "slotId is required for trigger_slot".to_string())?;
            trigger_slot_with_shared(slots, audio_engine, recording_engine, app_handle, &slot_id)
        }
        "stop_all_audio" => audio_engine.stop_all(),
        _ => Err(format!("unsupported command kind: {}", command.kind)),
    }
}

fn snapshot_state(
    slots: &Arc<Mutex<Vec<Slot>>>,
    timeline_state: &Arc<Mutex<crate::timeline::state::TimelineState>>,
    playback_engine: &Arc<Mutex<crate::timeline::playback::PlaybackEngine>>,
    recording_engine: &Arc<Mutex<crate::recording::engine::RecordingEngine>>,
    project_settings: &Arc<Mutex<crate::models::project::ProjectSettings>>,
    project_name: &Arc<Mutex<String>>,
) -> Result<RemoteStatePayload, String> {
    let slots = slots
        .lock()
        .map_err(|_| "failed to lock slots".to_string())?
        .clone();
    let (timeline_events, playhead_ms) = {
        let timeline = timeline_state
            .lock()
            .map_err(|_| "failed to lock timeline".to_string())?;
        (timeline.events.clone(), timeline.playhead_position_ms)
    };
    let playback_status = playback_engine
        .lock()
        .map_err(|_| "failed to lock playback engine".to_string())?
        .status();
    let recording_status = recording_engine
        .lock()
        .map_err(|_| "failed to lock recording engine".to_string())?
        .status();
    let settings = project_settings
        .lock()
        .map_err(|_| "failed to lock settings".to_string())?
        .clone();
    let project_name = project_name
        .lock()
        .map_err(|_| "failed to lock project name".to_string())?
        .clone();

    Ok(RemoteStatePayload {
        project_name,
        board_rows: settings.board_rows,
        board_columns: settings.board_columns,
        playhead_ms,
        playback_status,
        recording_status,
        slots,
        timeline_events,
    })
}

struct HttpRequest {
    method: String,
    path: String,
    query: String,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|err| format!("failed to set read timeout: {err}"))?;

    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    let mut header_end = None;
    loop {
        let read = stream.read(&mut temp).map_err(|err| format!("read failed: {err}"))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..read]);
        if let Some(index) = find_header_end(&buffer) {
            header_end = Some(index);
            break;
        }
        if buffer.len() > 64 * 1024 {
            return Err("request headers too large".to_string());
        }
    }

    let header_end = header_end.ok_or_else(|| "invalid request".to_string())?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = headers.lines();
    let request_line = lines.next().ok_or_else(|| "missing request line".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| "missing method".to_string())?
        .to_string();
    let target = request_parts
        .next()
        .ok_or_else(|| "missing request target".to_string())?
        .to_string();
    let (path, query) = target
        .split_once('?')
        .map(|(path, query)| (path.to_string(), query.to_string()))
        .unwrap_or((target, String::new()));

    let mut content_length = 0usize;
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    let body_start = header_end + 4;
    let mut body = buffer.get(body_start..).unwrap_or(&[]).to_vec();
    while body.len() < content_length {
        let read = stream.read(&mut temp).map_err(|err| format!("read body failed: {err}"))?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&temp[..read]);
    }
    body.truncate(content_length);

    Ok(HttpRequest {
        method,
        path,
        query,
        body,
    })
}

fn write_http_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .map_err(|err| format!("write headers failed: {err}"))?;
    stream
        .write_all(body)
        .map_err(|err| format!("write body failed: {err}"))?;
    Ok(())
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn local_ip_guess() -> String {
    let socket = UdpSocket::bind("0.0.0.0:0");
    if let Ok(socket) = socket {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

fn is_lan_or_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => addr.is_private() || addr.is_loopback() || addr.is_link_local(),
        IpAddr::V6(addr) => addr.is_loopback() || addr.is_unique_local() || addr.is_unicast_link_local(),
    }
}

fn remote_html() -> &'static str {
    r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SFX Board Remote</title>
    <link rel="stylesheet" href="/remote/styles.css" />
  </head>
  <body>
    <div id="app"></div>
    <script src="/remote/app.js"></script>
  </body>
</html>"#
}

fn remote_css() -> &'static str {
    r#":root{color-scheme:dark}
*{box-sizing:border-box}
body{margin:0;padding:10px;font-family:system-ui,-apple-system,Segoe UI,sans-serif;background:#080d1a;color:#f8fafc}
.app{display:grid;gap:10px;max-width:980px;margin:0 auto}
.top{display:flex;align-items:center;justify-content:space-between;gap:8px;flex-wrap:wrap}
.project{font-size:15px;font-weight:700}
.meta{font-size:12px;color:#9aa4b5}
.controls{display:flex;gap:8px}
.btn{border:1px solid #334155;border-radius:10px;padding:8px 12px;background:#192334;color:#f8fafc;font-weight:700}
.btn:active{transform:scale(.97)}
.grid{display:grid;gap:8px}
.slot-card{position:relative;height:102px;border:1px solid #334155;border-top:3px solid var(--slot-color);border-radius:10px;background:linear-gradient(160deg,#111827,#1a2235);color:#f8fafc;display:flex;flex-direction:column;justify-content:space-between;padding:8px;overflow:hidden}
.slot-loaded{cursor:pointer;transition:transform 120ms ease,border-color 120ms ease}
.slot-loaded:active{transform:scale(.98)}
.slot-playing{animation:slotPulse 260ms ease}
.slot-head{display:flex;justify-content:space-between;align-items:flex-start;gap:8px}
.slot-label{white-space:nowrap;overflow:hidden;text-overflow:ellipsis;font-size:14px;font-weight:700}
.slot-short{font-size:11px;border:1px solid #4b5563;border-radius:10px;min-width:30px;max-width:42px;aspect-ratio:1/1;color:#9ca3af;text-align:center;background:rgba(11,17,28,.9);display:inline-flex;align-items:center;justify-content:center;line-height:1}
.slot-short-active{color:#f9fafb;border-color:color-mix(in srgb,var(--slot-color) 55%,#334155);background:color-mix(in srgb,var(--slot-color) 28%,#111827)}
.slot-meta{margin-top:2px;font-size:11px;color:#9ca3af;display:flex;justify-content:space-between;gap:8px}
.slot-file{min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.slot-empty{justify-content:center;align-items:center;opacity:.55;border:1px dashed #334155;background:#111827;color:#9aa4b5}
@keyframes slotPulse{0%{box-shadow:0 0 0 0 color-mix(in srgb,var(--slot-color) 50%,transparent)}100%{box-shadow:0 0 0 12px transparent}}
@media (max-width:640px){body{padding:8px}.slot-card{height:96px;padding:7px}.slot-label{font-size:13px}}
"#
}

fn remote_app_js() -> &'static str {
    r#"(() => {
const token = new URLSearchParams(location.search).get('token') || '';

function esc(text){
  return String(text ?? '')
    .replaceAll('&','&amp;')
    .replaceAll('<','&lt;')
    .replaceAll('>','&gt;')
    .replaceAll('"','&quot;')
    .replaceAll("'","&#39;");
}

function fmtMs(ms){
  const n = Math.max(0, Math.floor(ms || 0));
  const m = String(Math.floor(n/60000)).padStart(2,'0');
  const s = String(Math.floor((n%60000)/1000)).padStart(2,'0');
  const mm = String(n%1000).padStart(3,'0');
  return `${m}:${s}.${mm}`;
}

async function send(payload){
  try{
    await fetch('/api/command?token='+encodeURIComponent(token),{
      method:'POST',
      headers:{'content-type':'application/json'},
      body:JSON.stringify(payload)
    });
  }catch{}
}

const app = document.getElementById('app');
let pressed = {};
let currentState = null;
let currentStatus = 'connecting...';
let stream = null;
let reconnectTimer = null;

function render(state, status){
  const rows = Math.max(1, state?.boardRows || 1);
  const cols = Math.max(1, state?.boardColumns || 1);
  const cap = rows * cols;
  const slotsByPos = new Map((state?.slots || []).map(s => [s.position, s]));
  let cards = '';
  for(let i=0;i<cap;i++){
    const slot = slotsByPos.get(i);
    if(!slot){
      cards += `<div class="slot-card slot-empty"><div class="slot-label">Empty</div><div class="slot-meta"><span>Slot ${i+1}</span></div></div>`;
      continue;
    }
    const color = slot.color || '#3a3a3a';
    const isPlaying = pressed[slot.id] ? 'slot-playing' : '';
    const file = String(slot.audioPath || '').split(/[\\/]/).filter(Boolean).pop() || slot.label || 'Untitled';
    cards += `<article class="slot-card slot-loaded ${isPlaying}" data-slot-id="${esc(slot.id)}" style="--slot-color:${esc(color)}">
      <div class="slot-head">
        <span class="slot-label" title="${esc(slot.label)}">${esc(slot.label || 'Untitled')}</span>
        <span class="slot-short ${(slot.shortcut ? 'slot-short-active' : '')}">${esc(slot.shortcut || '--')}</span>
      </div>
      <div class="slot-meta">
        <span class="slot-file" title="${esc(file)}">${esc(file)}</span>
        <span>${(slot.durationMs/1000).toFixed(2)}s</span>
      </div>
    </article>`;
  }

  const transport = status || 'connecting...';
  const stateStatus = state
    ? `${state.playbackStatus} | ${state.recordingStatus} | ${fmtMs(state.playheadMs)}`
    : 'waiting for state...';
  app.innerHTML = `
    <div class="app">
      <div class="top">
        <div>
          <div class="project">${esc(state?.projectName || 'SFX Board Remote')}</div>
          <div class="meta">${esc(transport)} | ${esc(stateStatus)}</div>
        </div>
        <div class="controls"><button class="btn" id="stopAllBtn">Stop All</button></div>
      </div>
      <div class="grid" style="grid-template-columns:repeat(${cols}, minmax(0,1fr))">${cards}</div>
    </div>
  `;

  const stopAllBtn = document.getElementById('stopAllBtn');
  if (stopAllBtn) {
    stopAllBtn.onclick = () => { void send({kind:'stop_all_audio'}); };
  }

  app.querySelectorAll('.slot-loaded').forEach((card) => {
    card.addEventListener('click', async () => {
      const slotId = card.getAttribute('data-slot-id');
      if (!slotId) return;
      pressed = {...pressed, [slotId]: true};
      render(state, status);
      setTimeout(() => {
        pressed = {...pressed, [slotId]: false};
        render(currentState, currentStatus);
      }, 220);
      await send({kind:'trigger_slot', slotId});
    });
  });
}

function connectStream(){
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  if (stream) {
    stream.close();
    stream = null;
  }
  currentStatus = 'connecting...';
  render(currentState, currentStatus);

  const url = '/api/events?token='+encodeURIComponent(token);
  stream = new EventSource(url);

  stream.addEventListener('open', () => {
    currentStatus = 'connected';
    render(currentState, currentStatus);
  });

  stream.addEventListener('state', (event) => {
    try{
      currentState = JSON.parse(event.data);
      currentStatus = 'connected';
      render(currentState, currentStatus);
    }catch{
      currentStatus = 'decode error';
      render(currentState, currentStatus);
    }
  });

  stream.addEventListener('error', () => {
    currentStatus = 'reconnecting...';
    render(currentState, currentStatus);
    if (stream) {
      stream.close();
      stream = null;
    }
    reconnectTimer = setTimeout(connectStream, 1200);
  });
}

window.addEventListener('beforeunload', () => {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  if (stream) {
    stream.close();
    stream = null;
  }
});

connectStream();
})();"#
}
