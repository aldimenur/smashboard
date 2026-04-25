# SFX Board - Task Breakdown for 5 Prompts

**Project:** SFX Board for Video Editors (Tauri + React)  
**Total Phases:** 5 Development Prompts  
**Approach:** Incremental delivery with working prototypes at each stage

---

## 📋 Development Strategy

### Philosophy
- Each prompt delivers a **working, testable increment**
- Build foundation → Add features → Polish → Export → Integrate
- Prioritize core workflow: Record → Edit → Export
- Test integration points early

### Deliverable Format per Prompt
- ✅ Working code (Rust + React)
- 📝 Implementation notes
- 🧪 Test scenarios
- 🐛 Known limitations
- ➡️ Next steps

---

## 🎯 PROMPT 1: Project Setup + Audio Engine + Basic Slot System

**Goal:** Establish Tauri project structure and prove audio playback works

### Tasks

#### 1.1 Project Initialization
- [ ] Create Tauri 2 project with React template
  ```bash
  npm create tauri-app
  # Choose: React + TypeScript + npm/pnpm
  ```
- [ ] Configure `tauri.conf.json`:
  - App name: "SFX Board"
  - Window size: 1024×600 min
  - Disable multiwindow
  - Windows-only build target
- [ ] Setup folder structure:
  ```
  src-tauri/
    ├── src/
    │   ├── main.rs
    │   ├── audio/
    │   │   ├── mod.rs
    │   │   ├── engine.rs      # Audio playback
    │   │   ├── decoder.rs     # WAV/MP3 decode
    │   │   └── mixer.rs       # Multi-instance mixing
    │   ├── models/
    │   │   ├── mod.rs
    │   │   └── slot.rs        # Slot struct
    │   └── commands/
    │       ├── mod.rs
    │       └── slot_commands.rs
    └── Cargo.toml
  
  src/
    ├── components/
    │   ├── SlotGrid.tsx
    │   └── SlotCard.tsx
    ├── hooks/
    │   └── useSlots.ts
    ├── types/
    │   └── index.ts
    └── App.tsx
  ```

#### 1.2 Rust Dependencies (Cargo.toml)
```toml
[dependencies]
tauri = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rodio = "0.18"           # Audio playback
hound = "3.5"            # WAV decode
minimp3 = "0.5"          # MP3 decode
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### 1.3 Audio Engine Implementation
- [ ] **Audio Decoder (`decoder.rs`)**
  - Function: `decode_wav(path: &Path) -> Result<AudioBuffer>`
  - Function: `decode_mp3(path: &Path) -> Result<AudioBuffer>`
  - Auto-detect format from file extension
  - Return: samples, sample_rate, channels, duration_ms

- [ ] **Audio Engine (`engine.rs`)**
  - Struct: `AudioEngine` dengan rodio `OutputStream`
  - Method: `play(buffer: AudioBuffer, gain: f32) -> PlaybackHandle`
  - Support multi-instance (no limit on concurrent playback)
  - Apply gain per playback instance
  - Thread-safe with `Arc<Mutex<>>`

- [ ] **Audio Mixer (`mixer.rs`)**
  - Basic additive mixing (rodio handles this)
  - Gain normalization to prevent clipping

#### 1.4 Slot Data Model
```rust
// models/slot.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Slot {
    pub id: String,
    pub label: String,
    pub audio_path: String,
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
    pub color: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}

impl Slot {
    pub fn new(audio_path: String, label: Option<String>) -> Result<Self> {
        // Decode audio to get duration
        // Generate color from path hash
        // Return slot with defaults
    }
}
```

#### 1.5 Tauri Commands for Slots
```rust
// commands/slot_commands.rs

#[tauri::command]
async fn add_slot(
    state: State<'_, AppState>,
    file_path: String,
    label: Option<String>,
) -> Result<Slot, String>

#[tauri::command]
async fn update_slot(
    state: State<'_, AppState>,
    slot_id: String,
    label: Option<String>,
    shortcut: Option<String>,
    gain: Option<f32>,
) -> Result<Slot, String>

#[tauri::command]
async fn delete_slot(
    state: State<'_, AppState>,
    slot_id: String,
) -> Result<(), String>

#[tauri::command]
async fn get_all_slots(
    state: State<'_, AppState>,
) -> Result<Vec<Slot>, String>

#[tauri::command]
async fn trigger_slot(
    state: State<'_, AppState>,
    slot_id: String,
) -> Result<(), String>
```

#### 1.6 React Frontend - Basic Slot Grid
- [ ] **TypeScript Types (`types/index.ts`)**
  ```typescript
  export interface Slot {
    id: string;
    label: string;
    audioPath: string;
    shortcut: string;
    gain: number;
    durationMs: number;
    color: string;
    createdAt: string;
  }
  ```

- [ ] **Slot Hook (`hooks/useSlots.ts`)**
  ```typescript
  export function useSlots() {
    const [slots, setSlots] = useState<Slot[]>([]);
    
    const loadSlots = async () => {
      const result = await invoke<Slot[]>('get_all_slots');
      setSlots(result);
    };
    
    const addSlot = async (filePath: string) => {
      const slot = await invoke<Slot>('add_slot', { filePath });
      setSlots(prev => [...prev, slot]);
    };
    
    const triggerSlot = async (slotId: string) => {
      await invoke('trigger_slot', { slotId });
    };
    
    return { slots, loadSlots, addSlot, triggerSlot };
  }
  ```

- [ ] **Slot Card Component (`SlotCard.tsx`)**
  - Display: label, shortcut badge, gain indicator
  - States: empty, loaded, playing (pulsing animation)
  - Click to trigger audio
  - Right-click menu: Edit, Delete

- [ ] **Slot Grid Component (`SlotGrid.tsx`)**
  - 8×8 grid layout (CSS Grid)
  - Render SlotCard for each position (0-63)
  - "Add Slot" button opens file picker
  - Use `@tauri-apps/plugin-dialog` for file selection

#### 1.7 Basic Dark Theme CSS
```css
:root {
  --bg-primary: #1A1A1A;
  --bg-secondary: #2A2A2A;
  --bg-tertiary: #3A3A3A;
  --text-primary: #FFFFFF;
  --text-secondary: #B0B0B0;
  --accent-primary: #3B82F6;
  --border-color: #404040;
}
```

### Testing Scenarios (Prompt 1)
1. ✅ Add WAV file → Slot appears dengan correct label/duration
2. ✅ Add MP3 file → Decodes successfully
3. ✅ Click slot → Audio plays immediately
4. ✅ Click same slot 3x rapidly → 3 instances play simultaneously
5. ✅ Adjust gain slider → Volume changes on next trigger
6. ✅ Delete slot → Removed from grid
7. ❌ Add corrupted audio file → Error message shown

### Deliverables (Prompt 1)
- ✅ Tauri app window opens
- ✅ 8×8 slot grid visible
- ✅ Can add WAV/MP3 files via file picker
- ✅ Clicking slot plays audio
- ✅ Multi-instance playback works
- ✅ Gain control functional

### Known Limitations
- No keyboard shortcuts yet
- No persistence (slots lost on restart)
- No visual feedback during playback
- No error handling for missing files

---

## 🎯 PROMPT 2: Recording System + Timeline Data Model + Global Shortcuts

**Goal:** Implement recording transport and capture timestamped events

### Tasks

#### 2.1 Recording Data Model
```rust
// models/recording.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RecordingStatus {
    Idle,
    Recording,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingSession {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub status: RecordingStatus,
    pub current_time_ms: f64,
    pub events_buffer: Vec<TimelineEvent>,
}

// models/timeline.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_id: String,
    pub time_ms: f64,
    pub slot_id: String,
    pub audio_path: String,
    pub label: String,
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
}
```

#### 2.2 Recording Engine
```rust
// recording/engine.rs
pub struct RecordingEngine {
    session: Option<RecordingSession>,
    start_time: Option<Instant>,
    paused_duration: Duration,
}

impl RecordingEngine {
    pub fn start(&mut self) -> Result<()> {
        // Create new session
        // Start timer
        // Set status to Recording
    }
    
    pub fn pause(&mut self) -> Result<()> {
        // Track paused duration
        // Set status to Paused
    }
    
    pub fn resume(&mut self) -> Result<()> {
        // Resume timer
        // Set status to Recording
    }
    
    pub fn stop(&mut self) -> Result<Vec<TimelineEvent>> {
        // Finalize session
        // Return events buffer
        // Reset state
    }
    
    pub fn capture_event(&mut self, slot: &Slot) -> Result<TimelineEvent> {
        // Calculate current_time_ms
        // Create TimelineEvent
        // Add to events_buffer
    }
    
    fn get_current_time_ms(&self) -> f64 {
        // start_time.elapsed() - paused_duration
    }
}
```

#### 2.3 Recording Commands
```rust
#[tauri::command]
async fn start_recording(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn pause_recording(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn resume_recording(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn stop_recording(
    state: State<'_, AppState>,
) -> Result<Vec<TimelineEvent>, String>

#[tauri::command]
async fn get_recording_status(
    state: State<'_, AppState>,
) -> Result<RecordingStatus, String>
```

#### 2.4 Recording Timer Thread
- [ ] Spawn background thread yang emit events setiap 16ms (60 Hz)
- [ ] Event: `recording-time-update` dengan payload `{ timeMs: number }`
- [ ] Frontend subscribe untuk update UI timer display

```rust
// In start_recording command:
let app_handle = app_handle.clone();
tokio::spawn(async move {
    while recording_active {
        let time_ms = get_current_time();
        app_handle.emit_all("recording-time-update", time_ms).ok();
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
});
```

#### 2.5 Global Shortcuts System
- [ ] Add dependency: `global-hotkey = "0.5"`
- [ ] Struct: `ShortcutManager`
  ```rust
  pub struct ShortcutManager {
      hotkey_manager: GlobalHotKeyManager,
      registered_shortcuts: HashMap<String, Slot>,
  }
  ```

- [ ] Register shortcuts when slot updated
  ```rust
  impl ShortcutManager {
      pub fn register(&mut self, slot: &Slot) -> Result<()> {
          let hotkey = HotKey::from_str(&slot.shortcut)?;
          self.hotkey_manager.register(hotkey)?;
          self.registered_shortcuts.insert(slot.id.clone(), slot.clone());
          Ok(())
      }
      
      pub fn handle_shortcut(&self, hotkey: HotKey) -> Option<Slot> {
          // Find slot by hotkey
          // Return slot to trigger
      }
  }
  ```

- [ ] Listen for global shortcut events in background thread
  ```rust
  let receiver = GlobalHotKeyEvent::receiver();
  std::thread::spawn(move || {
      loop {
          if let Ok(event) = receiver.recv() {
              // Trigger slot via shortcut
              // Emit event to frontend
          }
      }
  });
  ```

- [ ] Command: `set_global_shortcuts_enabled(enabled: bool)`

#### 2.6 Modified Slot Trigger Logic
```rust
#[tauri::command]
async fn trigger_slot(
    state: State<'_, AppState>,
    slot_id: String,
) -> Result<(), String> {
    let slot = /* get slot */;
    
    // Play audio
    audio_engine.play(&slot)?;
    
    // If recording, capture event
    if recording_engine.is_recording() {
        let event = recording_engine.capture_event(&slot)?;
    }
    
    // Emit event for UI feedback
    app_handle.emit_all("slot-triggered", slot_id)?;
    
    Ok(())
}
```

#### 2.7 React - Recording Transport UI
```typescript
// components/RecordingTransport.tsx
export function RecordingTransport() {
  const [status, setStatus] = useState<RecordingStatus>('Idle');
  const [timeMs, setTimeMs] = useState(0);
  const [eventCount, setEventCount] = useState(0);
  
  useEffect(() => {
    const unlisten = listen('recording-time-update', (event) => {
      setTimeMs(event.payload.timeMs);
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);
  
  const startRecording = async () => {
    await invoke('start_recording');
    setStatus('Recording');
  };
  
  const stopRecording = async () => {
    const events = await invoke<TimelineEvent[]>('stop_recording');
    setStatus('Idle');
    setEventCount(events.length);
    // TODO: Add events to timeline (next prompt)
  };
  
  return (
    <div className="recording-transport">
      <button onClick={startRecording}>⏺️ Record</button>
      <button onClick={pauseRecording}>⏸️ Pause</button>
      <button onClick={stopRecording}>⏹️ Stop</button>
      <div className="status">
        Status: {status} | Time: {formatTime(timeMs)} | Events: {eventCount}
      </div>
    </div>
  );
}
```

#### 2.8 Global Shortcuts UI
```typescript
// components/GlobalShortcutToggle.tsx
export function GlobalShortcutToggle() {
  const [enabled, setEnabled] = useState(false);
  
  const toggle = async () => {
    await invoke('set_global_shortcuts_enabled', { enabled: !enabled });
    setEnabled(!enabled);
  };
  
  return (
    <div className="global-shortcut-status">
      <button onClick={toggle}>
        {enabled ? '● Active' : '○ Inactive'}
      </button>
      Global Shortcuts
    </div>
  );
}
```

#### 2.9 Keyboard Shortcut Assignment UI
```typescript
// components/ShortcutInput.tsx
export function ShortcutInput({ slotId, currentShortcut }: Props) {
  const [recording, setRecording] = useState(false);
  const [shortcut, setShortcut] = useState(currentShortcut);
  
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!recording) return;
    
    e.preventDefault();
    const parts = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    parts.push(e.key.toUpperCase());
    
    const newShortcut = parts.join('+');
    setShortcut(newShortcut);
    setRecording(false);
    
    // Update slot
    invoke('update_slot', { slotId, shortcut: newShortcut });
  };
  
  return (
    <input
      value={recording ? 'Press keys...' : shortcut}
      onFocus={() => setRecording(true)}
      onBlur={() => setRecording(false)}
      onKeyDown={handleKeyDown}
      readOnly
    />
  );
}
```

### Testing Scenarios (Prompt 2)
1. ✅ Click Record → Status changes to "Recording", timer starts
2. ✅ Trigger slot during recording → Event captured dengan correct timestamp
3. ✅ Pause recording → Timer stops
4. ✅ Resume recording → Timer continues from paused position
5. ✅ Stop recording → Events returned, status back to Idle
6. ✅ Trigger 10 slots in 5 seconds → 10 events dengan accurate timestamps
7. ✅ Assign shortcut "Ctrl+1" → Can trigger via keyboard
8. ✅ Enable global shortcuts → Works when app not focused
9. ❌ Assign conflicting shortcut → Error/warning shown

### Deliverables (Prompt 2)
- ✅ Recording transport UI (Record/Pause/Stop buttons)
- ✅ Recording timer display (MM:SS.mmm)
- ✅ Event count display
- ✅ Keyboard shortcuts work (app-focused mode)
- ✅ Global shortcuts work (system-wide mode)
- ✅ Shortcut assignment UI in slot editor
- ✅ Events captured during recording
- ✅ Stop recording returns events array

### Known Limitations
- Events not yet displayed in timeline UI
- No timeline playback yet
- No persistence of recording sessions
- No undo/redo

---

## 🎯 PROMPT 3: Timeline Canvas + Timeline Editing + Playback

**Goal:** Visualize timeline events and implement editing operations

### Tasks

#### 3.1 Timeline State Management (Rust)
```rust
// timeline/state.rs
pub struct TimelineState {
    pub events: Vec<TimelineEvent>,
    pub total_duration_ms: f64,
    pub playhead_position_ms: f64,
}

impl TimelineState {
    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
        self.recalculate_duration();
    }
    
    pub fn delete_events(&mut self, event_ids: &[String]) {
        self.events.retain(|e| !event_ids.contains(&e.event_id));
        self.recalculate_duration();
    }
    
    pub fn update_event_time(&mut self, event_id: &str, new_time_ms: f64) {
        if let Some(event) = self.events.iter_mut().find(|e| e.event_id == event_id) {
            event.time_ms = new_time_ms.max(0.0);
        }
        self.recalculate_duration();
    }
    
    fn recalculate_duration(&mut self) {
        self.total_duration_ms = self.events.iter()
            .map(|e| e.time_ms + e.duration_ms)
            .fold(0.0, f64::max);
    }
    
    pub fn get_events_at_time(&self, time_ms: f64, lookahead_ms: f64) -> Vec<&TimelineEvent> {
        self.events.iter()
            .filter(|e| e.time_ms >= time_ms && e.time_ms < time_ms + lookahead_ms)
            .collect()
    }
}
```

#### 3.2 Timeline Commands
```rust
#[tauri::command]
async fn get_timeline_events(
    state: State<'_, AppState>,
) -> Result<Vec<TimelineEvent>, String>

#[tauri::command]
async fn add_timeline_event(
    state: State<'_, AppState>,
    slot_id: String,
    time_ms: f64,
) -> Result<TimelineEvent, String>

#[tauri::command]
async fn update_event_time(
    state: State<'_, AppState>,
    event_id: String,
    new_time_ms: f64,
) -> Result<(), String>

#[tauri::command]
async fn delete_timeline_events(
    state: State<'_, AppState>,
    event_ids: Vec<String>,
) -> Result<(), String>

#[tauri::command]
async fn duplicate_events(
    state: State<'_, AppState>,
    event_ids: Vec<String>,
) -> Result<Vec<TimelineEvent>, String>
```

#### 3.3 Timeline Playback Engine
```rust
// timeline/playback.rs
pub struct PlaybackEngine {
    status: PlaybackStatus,
    current_time_ms: f64,
    start_instant: Option<Instant>,
}

#[derive(Clone, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

impl PlaybackEngine {
    pub fn play(&mut self, from_time_ms: f64) {
        self.current_time_ms = from_time_ms;
        self.start_instant = Some(Instant::now());
        self.status = PlaybackStatus::Playing;
    }
    
    pub fn pause(&mut self) {
        self.status = PlaybackStatus::Paused;
    }
    
    pub fn stop(&mut self) {
        self.status = PlaybackStatus::Stopped;
    }
    
    pub fn get_current_time(&self) -> f64 {
        if self.status == PlaybackStatus::Playing {
            let elapsed = self.start_instant.unwrap().elapsed().as_secs_f64() * 1000.0;
            self.current_time_ms + elapsed
        } else {
            self.current_time_ms
        }
    }
}

// Playback thread
async fn playback_loop(
    app_handle: AppHandle,
    playback_engine: Arc<Mutex<PlaybackEngine>>,
    timeline_state: Arc<Mutex<TimelineState>>,
    audio_engine: Arc<Mutex<AudioEngine>>,
) {
    loop {
        let current_time = playback_engine.lock().unwrap().get_current_time();
        
        // Get events to trigger (100ms lookahead)
        let events = timeline_state.lock().unwrap()
            .get_events_at_time(current_time, 100.0);
        
        for event in events {
            // Trigger audio
            let slot = /* load slot from event */;
            audio_engine.lock().unwrap().play(&slot).ok();
            
            // Emit event triggered
            app_handle.emit_all("event-triggered", event.event_id).ok();
        }
        
        // Emit playhead update
        app_handle.emit_all("playhead-update", current_time).ok();
        
        tokio::time::sleep(Duration::from_millis(16)).await; // 60 Hz
    }
}
```

#### 3.4 Timeline Playback Commands
```rust
#[tauri::command]
async fn play_timeline(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn pause_timeline(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn stop_timeline(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn seek_timeline(
    state: State<'_, AppState>,
    time_ms: f64,
) -> Result<(), String>
```

#### 3.5 React - Timeline Canvas Component
```typescript
// components/Timeline/TimelineCanvas.tsx
interface TimelineEvent {
  eventId: string;
  timeMs: number;
  label: string;
  durationMs: number;
  color: string;
  track: number; // Vertical position (0-7)
}

export function TimelineCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [playheadMs, setPlayheadMs] = useState(0);
  const [zoom, setZoom] = useState(50); // ms per pixel
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [dragState, setDragState] = useState<DragState | null>(null);
  
  // Load events from backend
  useEffect(() => {
    invoke<TimelineEvent[]>('get_timeline_events').then(setEvents);
  }, []);
  
  // Listen for playhead updates
  useEffect(() => {
    const unlisten = listen('playhead-update', (event) => {
      setPlayheadMs(event.payload);
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);
  
  // Assign tracks to prevent overlap
  const assignTracks = (events: TimelineEvent[]): TimelineEvent[] => {
    const sorted = [...events].sort((a, b) => a.timeMs - b.timeMs);
    const tracks: { endTime: number }[] = [];
    
    return sorted.map(event => {
      // Find first available track
      const trackIndex = tracks.findIndex(t => t.endTime <= event.timeMs);
      
      if (trackIndex >= 0) {
        tracks[trackIndex].endTime = event.timeMs + event.durationMs;
        return { ...event, track: trackIndex };
      } else {
        tracks.push({ endTime: event.timeMs + event.durationMs });
        return { ...event, track: tracks.length - 1 };
      }
    });
  };
  
  // Render canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d')!;
    const width = canvas.width;
    const height = canvas.height;
    
    // Clear
    ctx.fillStyle = '#1A1A1A';
    ctx.fillRect(0, 0, width, height);
    
    // Draw grid lines (every 1 second)
    ctx.strokeStyle = '#2A2A2A';
    ctx.lineWidth = 1;
    for (let t = 0; t < width * zoom; t += 1000) {
      const x = t / zoom;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    
    // Draw events
    const trackedEvents = assignTracks(events);
    const trackHeight = 40;
    const trackSpacing = 8;
    
    trackedEvents.forEach(event => {
      const x = event.timeMs / zoom;
      const y = event.track * (trackHeight + trackSpacing) + 10;
      const w = event.durationMs / zoom;
      const h = trackHeight;
      
      // Draw block
      ctx.fillStyle = event.color;
      if (selectedIds.includes(event.eventId)) {
        ctx.strokeStyle = '#3B82F6';
        ctx.lineWidth = 2;
        ctx.strokeRect(x, y, w, h);
      }
      ctx.fillRect(x, y, w, h);
      
      // Draw label
      ctx.fillStyle = '#FFFFFF';
      ctx.font = '12px sans-serif';
      ctx.fillText(event.label, x + 4, y + 20);
    });
    
    // Draw playhead
    const playheadX = playheadMs / zoom;
    ctx.strokeStyle = '#FF0000';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(playheadX, 0);
    ctx.lineTo(playheadX, height);
    ctx.stroke();
    
  }, [events, playheadMs, zoom, selectedIds]);
  
  // Mouse handlers for selection and dragging
  const handleMouseDown = (e: React.MouseEvent) => {
    const rect = canvasRef.current!.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const timeMs = x * zoom;
    
    // Find clicked event
    const trackedEvents = assignTracks(events);
    const trackHeight = 40;
    const trackSpacing = 8;
    
    const clickedEvent = trackedEvents.find(event => {
      const eventX = event.timeMs / zoom;
      const eventY = event.track * (trackHeight + trackSpacing) + 10;
      const eventW = event.durationMs / zoom;
      const eventH = trackHeight;
      
      return x >= eventX && x <= eventX + eventW &&
             y >= eventY && y <= eventY + eventH;
    });
    
    if (clickedEvent) {
      // Start drag
      if (e.ctrlKey) {
        // Multi-select
        setSelectedIds(prev => 
          prev.includes(clickedEvent.eventId)
            ? prev.filter(id => id !== clickedEvent.eventId)
            : [...prev, clickedEvent.eventId]
        );
      } else {
        setSelectedIds([clickedEvent.eventId]);
      }
      
      setDragState({
        startX: x,
        startTimeMs: clickedEvent.timeMs,
        eventIds: selectedIds.includes(clickedEvent.eventId) 
          ? selectedIds 
          : [clickedEvent.eventId]
      });
    } else {
      // Clear selection
      setSelectedIds([]);
    }
  };
  
  const handleMouseMove = (e: React.MouseEvent) => {
    if (!dragState) return;
    
    const rect = canvasRef.current!.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const deltaMs = (x - dragState.startX) * zoom;
    
    // Update event times (optimistic UI)
    setEvents(prev => prev.map(event => 
      dragState.eventIds.includes(event.eventId)
        ? { ...event, timeMs: Math.max(0, event.timeMs + deltaMs) }
        : event
    ));
  };
  
  const handleMouseUp = async () => {
    if (!dragState) return;
    
    // Persist changes to backend
    for (const eventId of dragState.eventIds) {
      const event = events.find(e => e.eventId === eventId)!;
      await invoke('update_event_time', { eventId, newTimeMs: event.timeMs });
    }
    
    setDragState(null);
  };
  
  return (
    <canvas
      ref={canvasRef}
      width={1200}
      height={400}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      style={{ cursor: dragState ? 'grabbing' : 'default' }}
    />
  );
}
```

#### 3.6 Timeline Toolbar
```typescript
// components/Timeline/TimelineToolbar.tsx
export function TimelineToolbar() {
  const [zoom, setZoom] = useState(50);
  const [playbackStatus, setPlaybackStatus] = useState('Stopped');
  
  const play = async () => {
    await invoke('play_timeline');
    setPlaybackStatus('Playing');
  };
  
  const pause = async () => {
    await invoke('pause_timeline');
    setPlaybackStatus('Paused');
  };
  
  const stop = async () => {
    await invoke('stop_timeline');
    setPlaybackStatus('Stopped');
  };
  
  const deleteSelected = async () => {
    const ids = /* get selected IDs */;
    await invoke('delete_timeline_events', { eventIds: ids });
  };
  
  return (
    <div className="timeline-toolbar">
      <button onClick={play}>▶️ Play</button>
      <button onClick={pause}>⏸️ Pause</button>
      <button onClick={stop}>⏹️ Stop</button>
      <button onClick={deleteSelected}>🗑️ Delete</button>
      
      <div className="zoom-controls">
        <button onClick={() => setZoom(z => z * 0.8)}>−</button>
        <input 
          type="range" 
          min={10} 
          max={200} 
          value={zoom}
          onChange={e => setZoom(Number(e.target.value))}
        />
        <button onClick={() => setZoom(z => z * 1.2)}>+</button>
        <span>{zoom}ms/px</span>
      </div>
    </div>
  );
}
```

#### 3.7 Keyboard Shortcuts for Timeline
```typescript
// hooks/useTimelineKeyboard.ts
export function useTimelineKeyboard() {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === ' ') {
        e.preventDefault();
        invoke('play_timeline'); // Toggle play/pause
      }
      
      if (e.key === 'Escape') {
        invoke('stop_timeline');
      }
      
      if (e.key === 'Delete' || e.key === 'Backspace') {
        // Delete selected events
        const ids = /* get selected */;
        invoke('delete_timeline_events', { eventIds: ids });
      }
      
      if (e.key === 'ArrowLeft') {
        // Nudge -1 frame (33.33ms)
        const ids = /* get selected */;
        ids.forEach(id => {
          invoke('update_event_time', { 
            eventId: id, 
            newTimeMs: /* current - 33.33 */
          });
        });
      }
      
      if (e.key === 'ArrowRight') {
        // Nudge +1 frame
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);
}
```

### Testing Scenarios (Prompt 3)
1. ✅ Record session → Events appear in timeline canvas
2. ✅ Click event → Selected (blue outline)
3. ✅ Drag event → Moves horizontally, updates timestamp
4. ✅ Multiple overlapping events → Stacked vertically on different tracks
5. ✅ Click Play → Playhead advances, events trigger at correct times
6. ✅ Press Space → Timeline plays/pauses
7. ✅ Press Delete → Selected events removed
8. ✅ Press Arrow keys → Selected events nudge by 1 frame
9. ✅ Zoom in/out → Timeline scale changes
10. ✅ Stop playback → Playhead stops at current position

### Deliverables (Prompt 3)
- ✅ Timeline canvas rendering events
- ✅ Events displayed as colored blocks with labels
- ✅ Vertical track assignment to prevent overlap
- ✅ Playhead visualization
- ✅ Event selection (click + Ctrl-click for multi-select)
- ✅ Drag events to reposition
- ✅ Timeline playback with event triggering
- ✅ Zoom controls
- ✅ Delete events
- ✅ Nudge events with arrow keys

### Known Limitations
- No undo/redo yet
- No overlap counter badges
- No project save/load
- No export functionality
- Zoom resets canvas scroll position

---

## 🎯 PROMPT 4: Project Persistence + Undo/Redo + Export System

**Goal:** Save/load projects and implement export to WAV/MP3/JSON

### Tasks

#### 4.1 Project File Schema
```rust
// models/project.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub version: String,
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub settings: ProjectSettings,
    pub slots: Vec<Slot>,
    pub timeline: TimelineData,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectSettings {
    pub global_shortcuts_enabled: bool,
    pub audio_buffer_size: u32,
    pub frame_rate: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimelineData {
    pub events: Vec<TimelineEvent>,
    pub total_duration_ms: f64,
}

impl Project {
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let project: Project = serde_json::from_str(&json)?;
        Ok(project)
    }
    
    pub fn validate_audio_paths(&self) -> Vec<String> {
        let mut missing = Vec::new();
        
        for slot in &self.slots {
            if !Path::new(&slot.audio_path).exists() {
                missing.push(slot.audio_path.clone());
            }
        }
        
        missing
    }
}
```

#### 4.2 Project Management Commands
```rust
#[tauri::command]
async fn save_project(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String>

#[tauri::command]
async fn load_project(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Project, String>

#[tauri::command]
async fn validate_audio_paths(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String>

#[tauri::command]
async fn update_audio_path(
    state: State<'_, AppState>,
    old_path: String,
    new_path: String,
) -> Result<(), String>

#[tauri::command]
async fn autosave(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn get_autosave_path() -> Result<String, String>
```

#### 4.3 Autosave System
```rust
// autosave/mod.rs
pub struct AutosaveManager {
    last_save_time: Instant,
    interval_secs: u64,
}

impl AutosaveManager {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            last_save_time: Instant::now(),
            interval_secs,
        }
    }
    
    pub fn should_autosave(&self) -> bool {
        self.last_save_time.elapsed().as_secs() >= self.interval_secs
    }
    
    pub fn get_autosave_path() -> PathBuf {
        let app_data = std::env::var("APPDATA").unwrap();
        Path::new(&app_data)
            .join("SFXBoard")
            .join("autosave.sfxproj")
    }
}

// Background autosave thread
async fn autosave_loop(app_handle: AppHandle, state: Arc<Mutex<AppState>>) {
    let mut manager = AutosaveManager::new(120); // 2 minutes
    
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        if manager.should_autosave() {
            let project = state.lock().unwrap().to_project();
            let path = AutosaveManager::get_autosave_path();
            
            if let Ok(_) = project.save_to_file(&path) {
                manager.last_save_time = Instant::now();
                tracing::info!("Autosave completed");
            }
        }
    }
}
```

#### 4.4 Undo/Redo System
```rust
// undo/mod.rs
#[derive(Clone)]
pub enum UndoAction {
    AddEvent(TimelineEvent),
    DeleteEvents(Vec<TimelineEvent>),
    UpdateEventTime { event_id: String, old_time: f64, new_time: f64 },
    DuplicateEvents(Vec<TimelineEvent>),
}

pub struct UndoManager {
    undo_stack: Vec<UndoAction>,
    redo_stack: Vec<UndoAction>,
    max_depth: usize,
}

impl UndoManager {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }
    
    pub fn push(&mut self, action: UndoAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear(); // Clear redo on new action
    }
    
    pub fn undo(&mut self) -> Option<UndoAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<UndoAction> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }
}
```

#### 4.5 Undo/Redo Commands
```rust
#[tauri::command]
async fn undo(
    state: State<'_, AppState>,
) -> Result<(), String>

#[tauri::command]
async fn redo(
    state: State<'_, AppState>,
) -> Result<(), String>
```

#### 4.6 WAV Export Implementation
```rust
// export/wav.rs
use hound::{WavWriter, WavSpec};

pub fn export_timeline_to_wav(
    timeline: &TimelineState,
    slots: &[Slot],
    output_path: &Path,
) -> Result<()> {
    let sample_rate = 44100;
    let channels = 2;
    let bit_depth = 16;
    
    // Calculate total samples needed
    let duration_secs = (timeline.total_duration_ms / 1000.0) + 1.0; // +1s padding
    let total_samples = (duration_secs * sample_rate as f64) as usize;
    
    // Create stereo buffer
    let mut buffer_left = vec![0.0f32; total_samples];
    let mut buffer_right = vec![0.0f32; total_samples];
    
    // Mix each event into buffer
    for event in &timeline.events {
        let slot = slots.iter().find(|s| s.id == event.slot_id).unwrap();
        let audio_data = decode_audio(&slot.audio_path)?;
        
        let start_sample = (event.time_ms / 1000.0 * sample_rate as f64) as usize;
        let gain = event.gain;
        
        for (i, sample) in audio_data.samples.iter().enumerate() {
            let pos = start_sample + i;
            if pos >= total_samples { break; }
            
            // Mix with gain applied
            buffer_left[pos] += sample.0 * gain;
            buffer_right[pos] += sample.1 * gain;
        }
    }
    
    // Normalize to prevent clipping
    let peak = buffer_left.iter()
        .chain(buffer_right.iter())
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);
    
    let normalize_factor = if peak > 1.0 { 1.0 / peak } else { 1.0 };
    
    // Write WAV file
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(output_path, spec)?;
    
    for i in 0..total_samples {
        let left = (buffer_left[i] * normalize_factor * i16::MAX as f32) as i16;
        let right = (buffer_right[i] * normalize_factor * i16::MAX as f32) as i16;
        
        writer.write_sample(left)?;
        writer.write_sample(right)?;
    }
    
    writer.finalize()?;
    Ok(())
}
```

#### 4.7 MP3 Export Implementation
```rust
// export/mp3.rs
use mp3lame_encoder::{Builder, FlushNoGap};

pub fn export_timeline_to_mp3(
    timeline: &TimelineState,
    slots: &[Slot],
    output_path: &Path,
) -> Result<()> {
    // First create WAV in memory
    let wav_buffer = /* generate WAV buffer like above */;
    
    // Encode to MP3
    let mut encoder = Builder::new().expect("Create encoder");
    encoder.set_num_channels(2)?;
    encoder.set_sample_rate(44100)?;
    encoder.set_brate(mp3lame_encoder::Bitrate::Kbps320)?;
    encoder.set_quality(mp3lame_encoder::Quality::Best)?;
    
    let mut mp3_buffer = Vec::new();
    
    // Encode in chunks
    let chunk_size = 1152; // MP3 frame size
    for chunk in wav_buffer.chunks(chunk_size) {
        let encoded = encoder.encode(chunk)?;
        mp3_buffer.extend_from_slice(&encoded);
    }
    
    // Flush encoder
    let encoded = encoder.flush::<FlushNoGap>()?;
    mp3_buffer.extend_from_slice(&encoded);
    
    // Write to file
    std::fs::write(output_path, mp3_buffer)?;
    Ok(())
}
```

#### 4.8 JSON Export Implementation
```rust
// export/json.rs
#[derive(Serialize)]
pub struct TimelineExport {
    pub export_version: String,
    pub exported_at: DateTime<Utc>,
    pub project_name: String,
    pub frame_rate: u32,
    pub timeline: TimelineExportData,
    pub slots: Vec<SlotExportData>,
}

#[derive(Serialize)]
pub struct TimelineExportData {
    pub total_duration_ms: f64,
    pub total_duration_frames: u32,
    pub event_count: usize,
    pub events: Vec<EventExportData>,
}

#[derive(Serialize)]
pub struct EventExportData {
    pub event_id: String,
    pub time_ms: f64,
    pub time_frames: u32,
    pub time_formatted: String, // "MM:SS.mmm"
    pub label: String,
    pub audio_file: String,      // Filename only
    pub audio_path: String,      // Full path
    pub shortcut: String,
    pub gain: f32,
    pub duration_ms: f64,
    pub duration_frames: u32,
}

pub fn export_timeline_to_json(
    project: &Project,
    output_path: &Path,
) -> Result<()> {
    let frame_rate = project.settings.frame_rate;
    
    let export = TimelineExport {
        export_version: "1.0.0".to_string(),
        exported_at: Utc::now(),
        project_name: project.project_name.clone(),
        frame_rate,
        timeline: TimelineExportData {
            total_duration_ms: project.timeline.total_duration_ms,
            total_duration_frames: ms_to_frames(project.timeline.total_duration_ms, frame_rate),
            event_count: project.timeline.events.len(),
            events: project.timeline.events.iter().map(|e| EventExportData {
                event_id: e.event_id.clone(),
                time_ms: e.time_ms,
                time_frames: ms_to_frames(e.time_ms, frame_rate),
                time_formatted: format_time(e.time_ms),
                label: e.label.clone(),
                audio_file: Path::new(&e.audio_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                audio_path: e.audio_path.clone(),
                shortcut: e.shortcut.clone(),
                gain: e.gain,
                duration_ms: e.duration_ms,
                duration_frames: ms_to_frames(e.duration_ms, frame_rate),
            }).collect(),
        },
        slots: /* similar mapping */,
    };
    
    let json = serde_json::to_string_pretty(&export)?;
    std::fs::write(output_path, json)?;
    Ok(())
}

fn ms_to_frames(ms: f64, frame_rate: u32) -> u32 {
    (ms / 1000.0 * frame_rate as f64).round() as u32
}

fn format_time(ms: f64) -> String {
    let total_secs = ms / 1000.0;
    let mins = (total_secs / 60.0).floor() as u32;
    let secs = (total_secs % 60.0).floor() as u32;
    let millis = (ms % 1000.0).round() as u32;
    format!("{:02}:{:02}.{:03}", mins, secs, millis)
}
```

#### 4.9 Export Commands
```rust
#[tauri::command]
async fn export_audio_wav(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<(), String>

#[tauri::command]
async fn export_audio_mp3(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<(), String>

#[tauri::command]
async fn export_timeline_json(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<(), String>
```

#### 4.10 React - Save/Load UI
```typescript
// components/ProjectMenu.tsx
import { save, open } from '@tauri-apps/plugin-dialog';

export function ProjectMenu() {
  const [currentPath, setCurrentPath] = useState<string | null>(null);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  
  const saveProject = async () => {
    let path = currentPath;
    
    if (!path) {
      // First-time save
      path = await save({
        filters: [{ name: 'SFX Project', extensions: ['sfxproj'] }],
      });
      
      if (!path) return; // User cancelled
      setCurrentPath(path);
    }
    
    await invoke('save_project', { filePath: path });
    setHasUnsavedChanges(false);
    showToast('Project saved');
  };
  
  const saveProjectAs = async () => {
    const path = await save({
      filters: [{ name: 'SFX Project', extensions: ['sfxproj'] }],
    });
    
    if (!path) return;
    
    await invoke('save_project', { filePath: path });
    setCurrentPath(path);
    setHasUnsavedChanges(false);
  };
  
  const openProject = async () => {
    if (hasUnsavedChanges) {
      const confirm = await confirmDialog('Discard unsaved changes?');
      if (!confirm) return;
    }
    
    const path = await open({
      filters: [{ name: 'SFX Project', extensions: ['sfxproj'] }],
    });
    
    if (!path) return;
    
    try {
      const project = await invoke<Project>('load_project', { filePath: path });
      
      // Validate audio paths
      const missing = await invoke<string[]>('validate_audio_paths');
      
      if (missing.length > 0) {
        showMissingFilesDialog(missing);
      }
      
      setCurrentPath(path);
      setHasUnsavedChanges(false);
      // Update UI state with loaded project
      
    } catch (error) {
      showError('Failed to load project: ' + error);
    }
  };
  
  return (
    <div className="project-menu">
      <button onClick={saveProject}>
        💾 Save {hasUnsavedChanges && '*'}
      </button>
      <button onClick={saveProjectAs}>💾 Save As...</button>
      <button onClick={openProject}>📂 Open...</button>
    </div>
  );
}
```

#### 4.11 React - Export Dialog
```typescript
// components/ExportDialog.tsx
export function ExportDialog({ open, onClose }: Props) {
  const [exportWav, setExportWav] = useState(true);
  const [exportMp3, setExportMp3] = useState(true);
  const [exportJson, setExportJson] = useState(true);
  const [outputFolder, setOutputFolder] = useState('');
  const [prefix, setPrefix] = useState('MyProject_');
  const [exporting, setExporting] = useState(false);
  const [progress, setProgress] = useState(0);
  
  const handleExport = async () => {
    if (!outputFolder) {
      showError('Please select output folder');
      return;
    }
    
    setExporting(true);
    
    try {
      if (exportWav) {
        await invoke('export_audio_wav', {
          outputPath: `${outputFolder}\\${prefix}mixdown.wav`
        });
      }
      
      if (exportMp3) {
        await invoke('export_audio_mp3', {
          outputPath: `${outputFolder}\\${prefix}mixdown.mp3`
        });
      }
      
      if (exportJson) {
        await invoke('export_timeline_json', {
          outputPath: `${outputFolder}\\${prefix}timeline.json`
        });
      }
      
      showToast('Export complete!');
      onClose();
      
    } catch (error) {
      showError('Export failed: ' + error);
    } finally {
      setExporting(false);
    }
  };
  
  return (
    <Dialog open={open} onClose={onClose}>
      <h2>Export Project</h2>
      
      <div className="export-options">
        <label>
          <input type="checkbox" checked={exportWav} onChange={e => setExportWav(e.target.checked)} />
          WAV (lossless)
        </label>
        
        <label>
          <input type="checkbox" checked={exportMp3} onChange={e => setExportMp3(e.target.checked)} />
          MP3 (320kbps)
        </label>
        
        <label>
          <input type="checkbox" checked={exportJson} onChange={e => setExportJson(e.target.checked)} />
          JSON (timeline data)
        </label>
      </div>
      
      <div className="output-folder">
        <input value={outputFolder} readOnly />
        <button onClick={async () => {
          const folder = await open({ directory: true });
          if (folder) setOutputFolder(folder);
        }}>
          📁 Browse
        </button>
      </div>
      
      <div className="filename-prefix">
        <label>Filename Prefix:</label>
        <input value={prefix} onChange={e => setPrefix(e.target.value)} />
      </div>
      
      {exporting && (
        <div className="progress">
          <progress value={progress} max={100} />
          <span>Exporting... {progress}%</span>
        </div>
      )}
      
      <div className="actions">
        <button onClick={onClose}>Cancel</button>
        <button onClick={handleExport} disabled={exporting}>
          Export
        </button>
      </div>
    </Dialog>
  );
}
```

### Testing Scenarios (Prompt 4)
1. ✅ Save project → Creates .sfxproj file
2. ✅ Load project → Restores all slots and timeline events
3. ✅ Load project with missing audio → Shows warning dialog
4. ✅ Autosave triggers every 2 minutes
5. ✅ Restart app with autosave → Offers recovery
6. ✅ Edit timeline → Undo (Ctrl+Z) reverts change
7. ✅ Undo → Redo (Ctrl+Y) reapplies change
8. ✅ Export WAV → Creates valid audio file
9. ✅ Export MP3 → Creates valid 320kbps file
10. ✅ Export JSON → Contains all event data with frame numbers

### Deliverables (Prompt 4)
- ✅ Save project to .sfxproj file
- ✅ Load project from file
- ✅ Missing file validation and warning
- ✅ Autosave every 2 minutes
- ✅ Recovery prompt on startup
- ✅ Undo/Redo system (50 operations)
- ✅ Export WAV mixdown (44.1kHz, 16-bit)
- ✅ Export MP3 (320kbps)
- ✅ Export JSON timeline data
- ✅ Export dialog UI

### Known Limitations
- No keyboard shortcuts cheat sheet yet
- No overlap counter badges in timeline
- No visual polish/animations
- No comprehensive error messages

---

## 🎯 PROMPT 5: Polish + Keyboard Cheat Sheet + Final Integration + Testing

**Goal:** Complete remaining UI features, add cheat sheet, final bug fixes

### Tasks

#### 5.1 Keyboard Shortcuts Cheat Sheet
```typescript
// components/KeyboardShortcuts.tsx
export function KeyboardShortcuts({ open, onClose }: Props) {
  const shortcuts = [
    { category: 'Project', items: [
      { keys: 'Ctrl+S', action: 'Save Project' },
      { keys: 'Ctrl+Shift+S', action: 'Save Project As' },
      { keys: 'Ctrl+O', action: 'Open Project' },
      { keys: 'Ctrl+E', action: 'Export' },
    ]},
    { category: 'Recording', items: [
      { keys: 'R', action: 'Start Recording' },
      { keys: 'P', action: 'Pause/Resume Recording' },
      { keys: 'S', action: 'Stop Recording' },
    ]},
    { category: 'Timeline Playback', items: [
      { keys: 'Space', action: 'Play/Pause' },
      { keys: 'Esc', action: 'Stop' },
      { keys: 'Home', action: 'Rewind to Start' },
      { keys: 'End', action: 'Jump to End' },
    ]},
    { category: 'Editing', items: [
      { keys: 'Ctrl+Z', action: 'Undo' },
      { keys: 'Ctrl+Y', action: 'Redo' },
      { keys: 'Delete', action: 'Delete Selected Events' },
      { keys: 'Ctrl+A', action: 'Select All Events' },
      { keys: 'Ctrl+D', action: 'Duplicate Selected' },
      { keys: '←/→', action: 'Nudge ±1 frame' },
      { keys: 'Shift+←/→', action: 'Nudge ±10 frames' },
    ]},
    { category: 'View', items: [
      { keys: 'Ctrl++', action: 'Zoom In' },
      { keys: 'Ctrl+-', action: 'Zoom Out' },
      { keys: 'Ctrl+0', action: 'Zoom to Fit' },
    ]},
    { category: 'Help', items: [
      { keys: 'F1', action: 'Show This Dialog' },
    ]},
  ];
  
  return (
    <Dialog open={open} onClose={onClose} className="shortcuts-dialog">
      <h2>Keyboard Shortcuts</h2>
      
      <div className="shortcuts-grid">
        {shortcuts.map(category => (
          <div key={category.category} className="shortcut-category">
            <h3>{category.category}</h3>
            <table>
              <tbody>
                {category.items.map(item => (
                  <tr key={item.keys}>
                    <td className="shortcut-keys">
                      <kbd>{item.keys}</kbd>
                    </td>
                    <td className="shortcut-action">{item.action}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ))}
      </div>
      
      <button onClick={onClose}>Close</button>
    </Dialog>
  );
}

// Trigger with F1
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'F1') {
      e.preventDefault();
      setShortcutsOpen(true);
    }
  };
  
  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
}, []);
```

#### 5.2 Overlap Counter Badges
```typescript
// In TimelineCanvas.tsx - add overlap detection
const detectOverlaps = (events: TimelineEvent[]): Map<string, number> => {
  const overlapCounts = new Map<string, number>();
  
  for (let i = 0; i < events.length; i++) {
    const eventA = events[i];
    let count = 0;
    
    for (let j = 0; j < events.length; j++) {
      if (i === j) continue;
      
      const eventB = events[j];
      const aStart = eventA.timeMs;
      const aEnd = eventA.timeMs + eventA.durationMs;
      const bStart = eventB.timeMs;
      const bEnd = eventB.timeMs + eventB.durationMs;
      
      // Check overlap
      if (aStart < bEnd && aEnd > bStart) {
        count++;
      }
    }
    
    if (count > 0) {
      overlapCounts.set(eventA.eventId, count);
    }
  }
  
  return overlapCounts;
};

// In render loop - draw badges
const overlapCounts = detectOverlaps(events);

trackedEvents.forEach(event => {
  // ... draw block ...
  
  const overlapCount = overlapCounts.get(event.eventId);
  if (overlapCount && overlapCount > 0) {
    // Draw badge
    const badgeX = x + w - 20;
    const badgeY = y + 5;
    
    ctx.fillStyle = '#EF4444';
    ctx.beginPath();
    ctx.arc(badgeX, badgeY, 12, 0, Math.PI * 2);
    ctx.fill();
    
    ctx.fillStyle = '#FFFFFF';
    ctx.font = 'bold 10px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(`×${overlapCount}`, badgeX, badgeY + 3);
  }
});
```

#### 5.3 Title Bar with Unsaved Indicator
```typescript
// hooks/useWindowTitle.ts
export function useWindowTitle() {
  const [projectName, setProjectName] = useState('Untitled');
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  
  useEffect(() => {
    const title = `SFX Board - ${projectName}${hasUnsavedChanges ? '*' : ''}`;
    document.title = title;
    
    // Also update Tauri window title
    invoke('set_window_title', { title });
  }, [projectName, hasUnsavedChanges]);
  
  return { setProjectName, setHasUnsavedChanges };
}
```

#### 5.4 Status Bar
```typescript
// components/StatusBar.tsx
export function StatusBar() {
  const [eventCount, setEventCount] = useState(0);
  const [duration, setDuration] = useState(0);
  const [selectedCount, setSelectedCount] = useState(0);
  
  return (
    <div className="status-bar">
      <span>{eventCount} events</span>
      <span>|</span>
      <span>{formatTime(duration)}</span>
      {selectedCount > 0 && (
        <>
          <span>|</span>
          <span>{selectedCount} selected</span>
        </>
      )}
    </div>
  );
}
```

#### 5.5 Toast Notifications
```typescript
// components/Toast.tsx
import { useState, useEffect } from 'react';

let toastIdCounter = 0;

export function useToast() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  
  const showToast = (message: string, type: 'info' | 'success' | 'error' = 'info') => {
    const id = toastIdCounter++;
    setToasts(prev => [...prev, { id, message, type }]);
    
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, 3000);
  };
  
  return { toasts, showToast };
}

export function ToastContainer({ toasts }: { toasts: Toast[] }) {
  return (
    <div className="toast-container">
      {toasts.map(toast => (
        <div key={toast.id} className={`toast toast-${toast.type}`}>
          {toast.message}
        </div>
      ))}
    </div>
  );
}
```

#### 5.6 Missing Files Dialog
```typescript
// components/MissingFilesDialog.tsx
export function MissingFilesDialog({ files, onClose, onLocate }: Props) {
  return (
    <Dialog open={files.length > 0} onClose={onClose}>
      <h2>⚠️ Missing Audio Files</h2>
      
      <p>The following audio files could not be found:</p>
      
      <ul className="missing-files-list">
        {files.map(path => (
          <li key={path}>
            <code>{path}</code>
          </li>
        ))}
      </ul>
      
      <div className="actions">
        <button onClick={onLocate}>Locate Files</button>
        <button onClick={onClose}>Continue Anyway</button>
      </div>
    </Dialog>
  );
}
```

#### 5.7 Final CSS Polish
```css
/* Add smooth transitions */
.slot-card {
  transition: all 0.2s ease;
}

.slot-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.slot-card.playing {
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { box-shadow: 0 0 0 0 var(--accent-primary); }
  50% { box-shadow: 0 0 0 8px transparent; }
}

/* Timeline selection */
.timeline-event.selected {
  filter: brightness(1.2);
}

/* Button states */
button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

button:active {
  transform: scale(0.95);
}

/* Status bar */
.status-bar {
  display: flex;
  gap: 12px;
  padding: 8px 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-secondary);
}

/* Toast */
.toast-container {
  position: fixed;
  bottom: 20px;
  right: 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 1000;
}

.toast {
  padding: 12px 20px;
  border-radius: 8px;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.toast-success {
  background: var(--accent-success);
}

.toast-error {
  background: var(--accent-danger);
}
```

#### 5.8 Error Handling & Logging
```rust
// Setup tracing subscriber in main.rs
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("sfx_board=debug".parse().unwrap()))
        .init();
    
    tauri::Builder::default()
        // ...
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Add error logging to all commands
#[tauri::command]
async fn trigger_slot(
    state: State<'_, AppState>,
    slot_id: String,
) -> Result<(), String> {
    match trigger_slot_impl(state, slot_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to trigger slot: {}", e);
            Err(format!("Failed to trigger slot: {}", e))
        }
    }
}
```

#### 5.9 Integration Testing Checklist
- [ ] End-to-end workflow: Create project → Add slots → Record → Edit timeline → Export
- [ ] Global shortcuts work system-wide
- [ ] Autosave and recovery work correctly
- [ ] Undo/redo maintains consistent state
- [ ] Export produces valid audio files
- [ ] Missing file detection and relocation
- [ ] Keyboard shortcuts don't conflict
- [ ] Multi-select and drag operations
- [ ] Timeline zoom doesn't break rendering
- [ ] Memory usage stays reasonable (<500MB)

#### 5.10 Performance Optimizations
```typescript
// Debounce canvas redraws
const debouncedRender = useMemo(
  () => debounce(() => renderCanvas(), 16), // 60 FPS
  [events, playheadMs, zoom]
);

// Virtualize slot grid if >64 slots
const visibleSlots = useMemo(() => {
  // Only render slots in viewport
}, [slots, scrollPosition]);

// Throttle autosave checks
const throttledAutosave = useThrottle(checkAutosave, 10000); // 10s
```

### Testing Scenarios (Prompt 5)
1. ✅ Press F1 → Shortcuts cheat sheet opens
2. ✅ Overlapping events → Badge shows count
3. ✅ Unsaved changes → Asterisk in title bar
4. ✅ Export → Toast notification shows success
5. ✅ Load project with missing files → Dialog with list
6. ✅ All keyboard shortcuts work as documented
7. ✅ App runs smoothly with 500 events
8. ✅ Autosave doesn't interrupt workflow
9. ✅ Global shortcuts work while app minimized
10. ✅ Export creates valid files readable in other apps

### Deliverables (Prompt 5)
- ✅ Keyboard shortcuts cheat sheet (F1)
- ✅ Overlap counter badges in timeline
- ✅ Unsaved changes indicator in title
- ✅ Toast notifications system
- ✅ Status bar with stats
- ✅ Missing files dialog
- ✅ Smooth animations and transitions
- ✅ Error logging system
- ✅ Performance optimizations
- ✅ Full integration testing

### Known Limitations (Final)
- Windows-only (by design)
- No waveform visualization (by design)
- No drag-and-drop file import
- No MIDI controller support
- No VST plugin support

---

## 📦 Final Deliverables Summary

### After all 5 prompts, you will have:

✅ **Fully functional SFX Board application**
- 64-slot audio board dengan keyboard shortcuts
- Recording system dengan live event capture
- Timeline editing dengan drag, nudge, multi-select
- Timeline playback dengan precise event triggering
- Project save/load persistence
- Autosave dan crash recovery
- Undo/Redo system
- Export ke WAV/MP3/JSON
- Global shortcuts (system-wide)
- Keyboard shortcuts cheat sheet
- Dark theme UI dengan smooth animations

✅ **Production-ready executable**
- Portable .exe (no installer required)
- ~15-20MB file size
- Runs on Windows 10/11

✅ **Documentation**
- In-app keyboard shortcuts reference
- Code comments in Rust dan TypeScript
- PRD reference document

---

## 🎯 Next Steps After Completion

1. **User Testing**
   - Collect feedback dari target users
   - Identify UX pain points
   - Performance testing dengan large projects

2. **Bug Fixes & Optimization**
   - Address reported issues
   - Optimize export speed
   - Reduce memory usage

3. **Future Features (v2.0)**
   - Waveform visualization
   - Drag-and-drop file import
   - MIDI controller support
   - VST plugin support
   - macOS/Linux ports

---

**Estimated Development Time:**
- Prompt 1: 4-6 hours
- Prompt 2: 6-8 hours
- Prompt 3: 8-10 hours
- Prompt 4: 6-8 hours
- Prompt 5: 4-6 hours
- **Total: 28-38 hours**

**Prerequisites:**
- Rust installed (rustup)
- Node.js 18+ installed
- Windows 10/11 development machine
- VS Code dengan Rust dan TypeScript extensions
