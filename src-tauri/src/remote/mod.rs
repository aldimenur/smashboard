use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
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
                            let _ = handle_connection(
                                stream,
                                &token_for_thread,
                                &slots,
                                &audio_engine,
                                &recording_engine,
                                &timeline_state,
                                &playback_engine,
                                &project_settings,
                                &project_name,
                                &app_handle_for_thread,
                            );
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
    token: &str,
    slots: &Arc<Mutex<Vec<Slot>>>,
    audio_engine: &Arc<crate::audio::engine::AudioEngine>,
    recording_engine: &Arc<Mutex<crate::recording::engine::RecordingEngine>>,
    timeline_state: &Arc<Mutex<crate::timeline::state::TimelineState>>,
    playback_engine: &Arc<Mutex<crate::timeline::playback::PlaybackEngine>>,
    project_settings: &Arc<Mutex<crate::models::project::ProjectSettings>>,
    project_name: &Arc<Mutex<String>>,
    app_handle: &AppHandle,
) -> Result<(), String> {
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
            slots,
            timeline_state,
            playback_engine,
            recording_engine,
            project_settings,
            project_name,
        )?;
        let json = serde_json::to_vec(&payload).map_err(|err| format!("json encode failed: {err}"))?;
        return write_http_response(&mut stream, "200 OK", "application/json", &json);
    }

    if request.method == "POST" && request.path == "/api/command" {
        let command: RemoteCommandPayload =
            serde_json::from_slice(&request.body).map_err(|err| format!("invalid command payload: {err}"))?;
        execute_remote_command(&command, slots, audio_engine, recording_engine, app_handle)?;
        return write_http_response(&mut stream, "200 OK", "application/json", br#"{"ok":true}"#);
    }

    write_http_response(
        &mut stream,
        "404 Not Found",
        "application/json",
        br#"{"error":"not_found"}"#,
    )
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

fn remote_html() -> &'static str {
    r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SFX Board Remote</title>
    <style>
      body{font-family:system-ui,-apple-system,sans-serif;background:#111827;color:#f9fafb;margin:0;padding:12px}
      .top{display:flex;gap:8px;align-items:center;margin-bottom:10px;flex-wrap:wrap}
      .meta{font-size:12px;color:#9ca3af}
      .grid{display:grid;gap:8px}
      .slot{border:1px solid #374151;border-radius:10px;padding:10px;min-height:72px;display:flex;flex-direction:column;justify-content:space-between}
      .slot button{border:none;border-radius:8px;padding:8px;color:white;font-weight:700}
      .head{display:flex;justify-content:space-between;font-size:12px;color:#d1d5db}
      .label{font-size:14px;font-weight:700;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
      .controls button{background:#1f2937;color:#f9fafb;border:1px solid #374151;border-radius:8px;padding:8px 10px}
    </style>
  </head>
  <body>
    <div class="top">
      <strong id="project">SFX Board Remote</strong>
      <span class="meta" id="stateMeta">connecting...</span>
    </div>
    <div class="controls">
      <button onclick="sendCommand({kind:'stop_all_audio'})">Stop All</button>
    </div>
    <div id="grid" class="grid"></div>
    <script>
      const params=new URLSearchParams(location.search);
      const token=params.get('token')||'';
      const grid=document.getElementById('grid');
      const project=document.getElementById('project');
      const stateMeta=document.getElementById('stateMeta');

      async function sendCommand(payload){
        try{
          await fetch('/api/command?token='+encodeURIComponent(token),{
            method:'POST',headers:{'content-type':'application/json'},
            body:JSON.stringify(payload)
          });
        }catch{}
      }

      function color(slot){ return slot.color || '#374151'; }
      function esc(t){ return (t||'').replace(/[&<>'"]/g, c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])); }

      async function tick(){
        try{
          const res=await fetch('/api/state?token='+encodeURIComponent(token));
          if(!res.ok){ stateMeta.textContent='auth failed'; return; }
          const data=await res.json();
          project.textContent=data.projectName || 'SFX Board';
          stateMeta.textContent=`${data.playbackStatus} | ${data.recordingStatus} | ${Math.round(data.playheadMs)}ms`;
          grid.style.gridTemplateColumns=`repeat(${Math.max(1,data.boardColumns||1)}, minmax(0,1fr))`;
          const slotsByPos=new Map((data.slots||[]).map(s=>[s.position,s]));
          const cap=(data.boardRows||1)*(data.boardColumns||1);
          let html='';
          for(let i=0;i<cap;i++){
            const s=slotsByPos.get(i);
            if(!s){ html += `<div class="slot"><div class="head"><span>${i+1}</span><span>empty</span></div><div class="label">—</div></div>`; continue; }
            html += `<div class="slot" style="border-color:${color(s)}55;background:${color(s)}22">
              <div class="head"><span>${i+1}</span><span>${esc(s.shortcut||'--')}</span></div>
              <div class="label">${esc(s.label)}</div>
              <button style="background:${color(s)}" onclick="sendCommand({kind:'trigger_slot',slotId:'${esc(s.id)}'})">PLAY</button>
            </div>`;
          }
          grid.innerHTML=html;
        }catch{
          stateMeta.textContent='disconnected';
        }
      }
      setInterval(tick,300); tick();
    </script>
  </body>
</html>"#
}
