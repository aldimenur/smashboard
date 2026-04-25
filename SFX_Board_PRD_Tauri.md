# Product Requirements Document (PRD)
# SFX Board for Video Editors - Tauri Edition

**Version:** 2.0  
**Platform:** Windows Desktop  
**Tech Stack:** Rust + Tauri 2 + React  
**Last Updated:** 2026-04-25

---

## 1. Executive Summary

SFX Board adalah aplikasi desktop Windows untuk keyboard-first SFX performance dan timeline export workflows. Dibangun dengan Rust (backend/audio engine) dan React (frontend UI), aplikasi ini memungkinkan video editor untuk trigger audio clips secara live, merekam timestamped events, mengedit timeline dengan presisi frame-based, dan export dalam format audio (WAV/MP3) dan data (JSON).

**Core Value Proposition:**
- Capture SFX cues 10x lebih cepat vs manual timeline placement
- Frame-precise editing dengan keyboard shortcuts
- Lightweight executable tanpa installer overhead
- Export-ready untuk integration dengan NLE workflows

---

## 2. Target Users

### Primary Users
- **Video Editors** yang bekerja dengan banyak short SFX events (game videos, podcasts, video essays)
- **Sound Designers** yang perlu rapid prototyping SFX sequences
- **Content Creators** yang ingin live performance-style audio recording

### User Expertise Level
- Comfortable dengan keyboard shortcuts
- Familiar dengan basic audio concepts (gain, mixdown)
- Basic understanding timeline editing concepts

---

## 3. Technical Architecture

### 3.1 Technology Stack

#### Backend (Rust)
- **Framework:** Tauri 2.x
- **Audio Playback:** `rodio` (cross-platform audio playback)
- **Audio Decoding:** 
  - WAV: `hound` crate
  - MP3: `minimp3` or `symphonia`
- **Audio Export:**
  - WAV: `hound` 
  - MP3: `lame` bindings or `mp3lame-encoder`
- **Global Shortcuts:** `global-hotkey` crate
- **Storage:** `serde` + `serde_json` untuk project files
- **UUID Generation:** `uuid` crate
- **Logging:** `tracing` crate

#### Frontend (React)
- **Framework:** React 18+
- **Build Tool:** Vite (integrated dengan Tauri)
- **UI Library:** Custom components (no heavy UI framework)
- **State Management:** React Context + hooks
- **Styling:** CSS Modules atau Tailwind CSS (dark theme only)
- **Timeline Rendering:** HTML5 Canvas API
- **Icons:** Lucide React atau Phosphor Icons

#### Development Tools
- **Package Manager:** pnpm atau npm
- **Rust Toolchain:** stable channel
- **Target:** x86_64-pc-windows-msvc

### 3.2 Application Architecture

```
┌─────────────────────────────────────────┐
│          React Frontend (UI)            │
│  ┌─────────────────────────────────┐   │
│  │  Components                     │   │
│  │  - SFX Board                    │   │
│  │  - Timeline Canvas              │   │
│  │  - Transport Controls           │   │
│  │  - Export Dialog                │   │
│  └─────────────────────────────────┘   │
└──────────────┬──────────────────────────┘
               │ IPC (Tauri Commands)
               │
┌──────────────▼──────────────────────────┐
│         Rust Backend (Tauri)            │
│  ┌─────────────────────────────────┐   │
│  │  Audio Engine                   │   │
│  │  - Multi-instance playback      │   │
│  │  - Mixing & gain control        │   │
│  │  - Real-time triggering         │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Recording Engine               │   │
│  │  - Timestamp capture            │   │
│  │  - Event queue management       │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Timeline Engine                │   │
│  │  - Event CRUD operations        │   │
│  │  - Playback scheduling          │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Export Engine                  │   │
│  │  - WAV/MP3 mixdown              │   │
│  │  - JSON serialization           │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Project Management             │   │
│  │  - Save/Load/Autosave           │   │
│  │  - File validation              │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Global Shortcut Manager        │   │
│  │  - System-wide hotkey capture   │   │
│  │  - Conflict detection           │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### 3.3 IPC Communication Pattern

**Frontend → Backend (Commands)**
```javascript
// Tauri invoke commands
await invoke('load_audio_file', { path: '/path/to/audio.wav' })
await invoke('trigger_slot', { slotId: 'uuid' })
await invoke('start_recording')
```

**Backend → Frontend (Events)**
```javascript
// Event listeners
listen('recording-started', (event) => { /* update UI */ })
listen('playhead-update', (event) => { /* move playhead */ })
listen('audio-triggered', (event) => { /* visual feedback */ })
```

---

## 4. Core Features

### 4.1 SFX Board (Slot Management)

#### 4.1.1 Slot Properties
Each slot contains:
- **ID:** UUID (auto-generated)
- **Label:** String (user-defined, max 32 chars)
- **Audio File Path:** Absolute path to WAV/MP3 file
- **Shortcut:** Keyboard shortcut string (e.g., "Ctrl+Shift+A")
- **Gain:** Float 0.0 - 2.0 (default 1.0, representing 0% - 200%)
- **Duration:** Auto-detected dari audio file (milliseconds)
- **Color:** Auto-assigned hex color untuk visual identification

#### 4.1.2 Slot Operations
- **Add Slot:** Open file picker → select WAV/MP3 → auto-create slot
- **Edit Slot:** Modify label, shortcut, gain
- **Delete Slot:** Remove slot (confirm jika slot memiliki events di timeline)
- **Duplicate Slot:** Clone slot dengan suffix " (copy)"
- **Reorder Slots:** Drag-and-drop reordering (visual only, tidak affect IDs)

#### 4.1.3 Slot Grid Display
- Grid layout: 5 columns × 5 rows = 25 slots max
- Visual states:
  - **Empty:** Dashed border dengan "+" icon
  - **Loaded:** Solid border, label, shortcut badge, waveform icon
  - **Active (playing):** Pulsing glow effect
  - **Selected:** Highlighted border
  - **Missing audio:** Red warning icon overlay

#### 4.1.4 Shortcut Assignment
- Setiap slot punya custom shortcut sendiri (editable per-slot)
- Click slot → press any keyboard combo
- Supported modifiers: Ctrl, Alt, Shift, combinations
- Conflict detection: warn jika shortcut sudah digunakan slot lain
- Shortcut scope: keyboard shortcuts dipakai khusus untuk trigger SFX slot
- Tidak ada shortcut keyboard khusus untuk playback/timeline/project context

#### 4.1.5 Global Shortcut Status Indicator
- Status bar widget showing:
  - **Active (Green):** Global shortcuts are captured system-wide
  - **Inactive (Gray):** Shortcuts only work when app is focused
  - Toggle button untuk enable/disable global mode
- Visual notification saat slot triggered via global shortcut

### 4.2 Recording System

#### 4.2.1 Recording Transport States
- **IDLE:** Not recording, ready to start
- **RECORDING:** Active recording, capturing triggers
- **PAUSED:** Recording paused, can resume
- **STOPPED:** Recording finished, events saved to timeline

#### 4.2.2 Transport Controls
- **Start Recording (button):**
  - Clear previous recording session (dengan confirm dialog)
  - Start internal timer at 0ms
  - Transition to RECORDING state
  - Enable trigger capture
  
- **Pause/Resume (button):**
  - Pause timer without clearing data
  - Resume timer from paused position
  
- **Stop Recording (button):**
  - Finalize recording session
  - Commit all captured events to timeline
  - Transition to IDLE state
  - Show summary: "Recorded X events in MM:SS.mmm"

#### 4.2.3 Event Capture During Recording
When slot triggered (via shortcut or click):
1. Capture current timer position (ms precision)
2. Create event object:
   ```json
   {
     "eventId": "uuid-v4",
     "timeMs": 5420.5,
     "slotId": "slot-uuid",
     "audioPath": "/absolute/path/to/audio.wav",
     "label": "Explosion",
     "shortcut": "Ctrl+1",
     "gain": 0.8,
     "durationMs": 1250
   }
   ```
3. Add to recording buffer
4. Play audio immediately (real-time feedback)
5. Visual feedback: pulse effect on triggered slot

#### 4.2.4 Multi-Instance Playback
- Slots dapat di-trigger multiple kali simultaneously
- Setiap trigger = independent audio instance
- Mixing handled oleh audio engine (Rust rodio)
- Gain per-slot diterapkan pada semua instances

#### 4.2.5 Recording Session Metadata
```json
{
  "sessionId": "uuid",
  "startedAt": "2026-04-25T10:30:00Z",
  "durationMs": 125340,
  "eventCount": 47,
  "status": "completed"
}
```

### 4.3 Timeline System

#### 4.3.1 Timeline Canvas (HTML5 Canvas)

**Visual Elements:**
- **Horizontal time axis:** 0ms → total duration
- **Vertical tracks:** Auto-stacked untuk prevent complete overlap
- **Event blocks:** Colored rectangles (per audio file)
- **Playhead:** Red vertical line dengan timestamp label
- **Time markers:** Gridlines setiap 1 second (adjustable dengan zoom)
- **Selection indicator:** Blue outline pada selected events

**Rendering Specs:**
- Timeline height: Variable based on track count (min 120px, max 800px)
- Event block height: 40px
- Track spacing: 8px vertical gap
- Minimum event width: 20px (untuk very short audio)
- Canvas updates: 60 FPS during playback, on-demand saat editing

**Color Scheme:**
- Blocks: Consistent color per audio file (hash-based color generation)
- Overlap indicator: Small badge dengan angka overlap count
- Selected block: Blue outline (2px)
- Playhead: Red (#FF0000)
- Grid lines: Dark gray (#2A2A2A)
- Background: Darker gray (#1A1A1A)

#### 4.3.2 Timeline Controls

**Zoom Controls:**
- **Zoom In (Ctrl + Plus):** Increase time scale (show more detail)
- **Zoom Out (Ctrl + Minus):** Decrease time scale (fit more time)
- **Zoom to Fit (Ctrl + 0):** Scale timeline to show all events
- **Zoom Levels:** 10 preset levels (10ms/px → 1000ms/px)
- Visual zoom slider di toolbar

**Selection:**
- **Click:** Select single event
- **Ctrl + Click:** Multi-select (add/remove from selection)
- **Shift + Click:** Range select (select all between first and last)
- **Drag Box:** Click empty space + drag untuk box select
- **Select All (Ctrl + A):** Select semua events di timeline

**Pan/Scroll:**
- **Horizontal scroll:** Mouse wheel atau drag timeline
- **Auto-scroll during playback:** Playhead stays centered
- **Snap to playhead (Home key):** Center view on playhead

#### 4.3.3 Timeline Editing Operations

**Move/Reposition:**
- **Drag selected events:** Click + drag untuk move horizontally
- **Nudge Earlier (Left Arrow):** Move -1 frame
- **Nudge Later (Right Arrow):** Move +1 frame
- **Fine Nudge (Shift + Arrows):** Move ±10 frames
- Frame duration: 1/30 second = ~33.33ms (standard video frame)
- Events can overlap other events (no collision prevention)

**Delete:**
- **Delete selected (Delete key atau Backspace):** Remove dari timeline
- **Confirm dialog jika >5 events selected**
- Cannot delete while recording or playing

**Cut (Ctrl + X):**
- Remove selected events and store in clipboard (in-memory)
- Paste not implemented (future feature)

**Duplicate (Ctrl + D):**
- Clone selected events at same position + 100ms offset
- Useful untuk rapid repeat patterns

**Add Event Manually:**
- Click "Add Event" button di toolbar
- Opens slot selector dialog
- Event placed at current playhead position
- Fallback: place at timeline start (0ms)

#### 4.3.4 Overlap Handling

**Visual Overlap Indicator:**
When events overlap in time:
- Stack events vertically (auto-track assignment)
- If vertical space exhausted (>8 tracks):
  - Show overlap badge dengan angka count
  - Badge positioned at top-right corner of block
  - Example: "×3" means 3 events overlap at this time

**Collision Calculations:**
```
Event A: [500ms → 1500ms]
Event B: [1200ms → 2000ms]
→ Overlap detected: [1200ms → 1500ms]
→ Assign to different vertical tracks
```

**Maximum Track Count:** 8 tracks
- If exceeded, events share tracks dengan overlap indicators

#### 4.3.5 Timeline Metadata Display

**Event Tooltip (hover):**
```
Label: Explosion
Time: 00:05.420
Duration: 1.25s
Shortcut: Ctrl+1
Audio: explosion.wav
```

**Timeline Stats Bar:**
- Total events: 47
- Total duration: 02:05.340
- Selected: 3 events
- Zoom level: 50ms/px

#### 4.3.6 Undo/Redo System

**Supported Operations:**
- Move events
- Delete events
- Add events
- Edit event properties
- NOT recording sessions (too complex)

**Implementation:**
- Command pattern dengan history stack
- Max history depth: 50 operations
- Undo (Ctrl + Z)
- Redo (Ctrl + Y)
- Clear history on project load

### 4.4 Timeline Playback

#### 4.4.1 Playback Transport

**States:**
- **STOPPED:** Playhead at last stop position
- **PLAYING:** Playhead advancing, events triggering
- **PAUSED:** Playhead frozen, can resume

**Controls:**
- **Play (button):** Start from current playhead position
- **Pause (button):** Freeze playhead
- **Stop (button):** Stop playback, playhead stays at current position
- **Rewind to Start (button):** Move playhead to 0ms
- **Jump to End (button):** Move playhead to last event + 1s

#### 4.4.2 Event Triggering During Playback

**Scheduling Algorithm:**
1. Get all events within lookahead window (100ms ahead)
2. Sort by timestamp ascending
3. Schedule audio trigger at precise time
4. Visual feedback: flash event block when triggered

**Timing Precision:**
- Target: ±5ms accuracy
- Uses Rust high-precision timers
- Compensates for audio buffer latency

**Playhead Updates:**
- Update frequency: 60 Hz (every ~16.67ms)
- Emit IPC event to frontend untuk smooth rendering
- Payload: `{ currentTimeMs: 5420.5 }`

#### 4.4.3 Playback Priority

**Conflict Resolution:**
- Recording transport has priority over timeline playback
- Starting recording auto-stops timeline playback
- Visual warning: "Timeline playback stopped (recording active)"

### 4.5 Project Management

#### 4.5.1 Project File Structure

**Single JSON File:**
```json
{
  "version": "1.0.0",
  "projectName": "My SFX Session",
  "createdAt": "2026-04-25T10:30:00Z",
  "modifiedAt": "2026-04-25T12:45:30Z",
  "settings": {
    "globalShortcutsEnabled": true,
    "audioBufferSize": 512,
    "frameRate": 30
  },
  "slots": [
    {
      "id": "uuid-1",
      "label": "Explosion",
      "audioPath": "C:\\Audio\\explosion.wav",
      "shortcut": "Ctrl+1",
      "gain": 0.8,
      "durationMs": 1250,
      "color": "#FF6B6B"
    }
  ],
  "timeline": {
    "events": [
      {
        "eventId": "uuid-event-1",
        "timeMs": 5420.5,
        "slotId": "uuid-1",
        "audioPath": "C:\\Audio\\explosion.wav",
        "label": "Explosion",
        "shortcut": "Ctrl+1",
        "gain": 0.8,
        "durationMs": 1250
      }
    ],
    "totalDurationMs": 125340
  }
}
```

**File Extension:** `.sfxproj`

#### 4.5.2 Save Project Workflow

**First-Time Save (Ctrl + S):**
1. Open Windows file save dialog
2. User selects folder + filename
3. Validate folder write permissions
4. Serialize project state to JSON
5. Write to disk
6. Update app title bar: "ProjectName.sfxproj"
7. Mark project as "saved" (no unsaved changes indicator)

**Subsequent Saves:**
- Save to same file path without dialog
- Show toast notification: "Project saved"

**Save As (Ctrl + Shift + S):**
- Always open save dialog
- Allows saving to different location/name

**Unsaved Changes Indicator:**
- Asterisk (*) di title bar jika ada unsaved changes
- Confirm dialog on app close jika unsaved

#### 4.5.3 Open Project Workflow

**Open (Ctrl + O):**
1. If current project unsaved → confirm discard changes
2. Open Windows file open dialog (.sfxproj filter)
3. Load JSON and validate schema
4. Validate audio file paths:
   - If missing → show warning list
   - Option: "Locate missing files" atau "Continue anyway"
5. Reconstruct application state
6. Update UI (slots, timeline, settings)
7. Update title bar dengan project name

**Recent Project:** Tidak ada (out of scope)

**Open on Startup:**
- Empty state (no project loaded)
- Option: "New Project" atau "Open Project"

#### 4.5.4 Autosave System

**Autosave Rules:**
- Trigger: Every 2 minutes jika ada unsaved changes
- Location: `%APPDATA%/SFXBoard/autosave.sfxproj`
- Single autosave file (overwrite previous)
- Autosave tidak count as "saved project"

**Recovery on Startup:**
- Check if autosave exists dan newer than last project save
- Show dialog: "Recover unsaved work? (Last autosave: 2 mins ago)"
- Options: "Recover" atau "Discard"

#### 4.5.5 File Path Validation

**On Project Load:**
- Check setiap `audioPath` di slots dan events
- Build list of missing files
- Show validation report:
  ```
  Missing Audio Files:
  - C:\Audio\explosion.wav (used by 5 events)
  - C:\Audio\whoosh.mp3 (Slot: Whoosh)
  
  [Locate Files] [Continue Anyway] [Cancel]
  ```

**Locate Files Flow:**
- User browses untuk each missing file
- Update all references to new path
- Save corrected paths to project

### 4.6 Export System

#### 4.6.1 Export Dialog

**Triggered by:** Ctrl + E atau menu Export

**Dialog Layout:**
```
┌─────────────────────────────────────┐
│ Export Project                      │
├─────────────────────────────────────┤
│ Audio Export:                       │
│ ☑ WAV (lossless)                    │
│ ☑ MP3 (compressed, 320kbps)         │
│                                     │
│ Data Export:                        │
│ ☑ JSON (timeline data)              │
│                                     │
│ Output Folder:                      │
│ [C:\Projects\MyProject\Export] [📁] │
│                                     │
│ Filename Prefix:                    │
│ [MyProject_]                        │
│                                     │
│          [Cancel]  [Export]         │
└─────────────────────────────────────┘
```

**Export Files Generated:**
- `MyProject_mixdown.wav`
- `MyProject_mixdown.mp3`
- `MyProject_timeline.json`

#### 4.6.2 Audio Mixdown Export

**WAV Export:**
- Sample rate: 44.1kHz (hardcoded, tidak configurable)
- Bit depth: 16-bit (hardcoded)
- Channels: Stereo
- Format: PCM uncompressed

**MP3 Export:**
- Bitrate: 320kbps CBR (hardcoded)
- Sample rate: 44.1kHz
- Channels: Stereo
- Encoder: LAME

**Mixdown Algorithm:**
1. Determine total duration (last event end time + 1s padding)
2. Create empty audio buffer (duration × sample rate)
3. For each timeline event:
   - Load audio file
   - Apply slot gain
   - Mix into buffer at event timestamp
   - Handle multi-instance overlaps (additive mixing)
4. Normalize to prevent clipping (peak detection)
5. Export to WAV/MP3 file

**Progress Indicator:**
- Show progress bar: "Mixing audio... 47/47 events"
- Estimated time remaining

#### 4.6.3 JSON Data Export

**Export Structure:**
```json
{
  "exportVersion": "1.0.0",
  "exportedAt": "2026-04-25T14:30:00Z",
  "projectName": "My SFX Session",
  "frameRate": 30,
  "timeline": {
    "totalDurationMs": 125340,
    "totalDurationFrames": 3760,
    "eventCount": 47,
    "events": [
      {
        "eventId": "uuid-event-1",
        "timeMs": 5420.5,
        "timeFrames": 163,
        "timeFormatted": "00:05.420",
        "label": "Explosion",
        "audioFile": "explosion.wav",
        "audioPath": "C:\\Audio\\explosion.wav",
        "shortcut": "Ctrl+1",
        "gain": 0.8,
        "durationMs": 1250,
        "durationFrames": 38
      }
    ]
  },
  "slots": [
    {
      "id": "uuid-1",
      "label": "Explosion",
      "audioFile": "explosion.wav",
      "audioPath": "C:\\Audio\\explosion.wav",
      "shortcut": "Ctrl+1",
      "gain": 0.8,
      "usageCount": 5
    }
  ]
}
```

**Key Features:**
- **Frame-based timing:** Include both ms dan frame number
- **Formatted timestamps:** Human-readable "MM:SS.mmm"
- **Audio file references:** Both filename dan absolute path
- **Usage stats:** How many times each slot was used
- **NO embedded audio data:** Only file references

#### 4.6.4 Export Validation

**Pre-Export Checks:**
- All audio files still exist at referenced paths
- Timeline has at least 1 event (warn if empty)
- Output folder is writable

**Error Handling:**
- Show detailed error messages
- Partial export option jika some audio files missing
- Retry mechanism untuk failed exports

**Success Notification:**
- Toast: "Export complete! 3 files created."
- Option to open output folder

### 4.7 Keyboard Shortcuts Reference

#### 4.7.1 Shortcuts Cheat Sheet

**Accessible via:** Help menu button

**Dialog Display:**
```
┌──────────────────────────────────────────┐
│ Keyboard Shortcuts                       │
├──────────────────────────────────────────┤
│ SFX BOARD                                │
│ Assigned per slot  Trigger SFX           │
│ Click Slot         Trigger SFX (mouse)   │
│                                          │
│                 [Close]                  │
└──────────────────────────────────────────┘
```

**Features:**
- Menjelaskan bahwa keyboard shortcut difokuskan untuk trigger SFX slot

---

## 5. UI/UX Design Specifications

### 5.1 Color Palette (Dark Theme Only)

```css
/* Background Colors */
--bg-primary: #1A1A1A;      /* Main background */
--bg-secondary: #2A2A2A;    /* Panel backgrounds */
--bg-tertiary: #3A3A3A;     /* Hover states */

/* Text Colors */
--text-primary: #FFFFFF;    /* Main text */
--text-secondary: #B0B0B0;  /* Secondary text */
--text-muted: #707070;      /* Disabled text */

/* Accent Colors */
--accent-primary: #3B82F6;  /* Blue - primary actions */
--accent-success: #10B981;  /* Green - success states */
--accent-warning: #F59E0B;  /* Yellow - warnings */
--accent-danger: #EF4444;   /* Red - errors, playhead */

/* UI Elements */
--border-color: #404040;
--selection-bg: rgba(59, 130, 246, 0.2);
--hover-bg: rgba(255, 255, 255, 0.05);
```

### 5.2 Layout Structure

```
┌────────────────────────────────────────────────────────┐
│ Title Bar: SFXBoard - MyProject.sfxproj*              │
├────────────────────────────────────────────────────────┤
│ Menu: File  Edit  View  Help    [Global: ●Active]     │
├────────────────────────────────────────────────────────┤
│ ┌────────────────────────┬─────────────────────────┐  │
│ │  SFX Board (Slots)     │  Recording Transport    │  │
│ │  5×5 Grid              │  ┌────────────────────┐ │  │
│ │  [Slot1] [Slot2]...    │  │ ⏺️ Rec  ⏸ Pause   │ │  │
│ │  [Slot9] [Slot10]...   │  │ ⏹️ Stop 🔄 New    │ │  │
│ │                        │  │                    │ │  │
│ │                        │  │ Status: IDLE       │ │  │
│ │                        │  │ Time: 00:00.000    │ │  │
│ │                        │  │ Events: 0          │ │  │
│ │                        │  └────────────────────┘ │  │
│ └────────────────────────┴─────────────────────────┘  │
├────────────────────────────────────────────────────────┤
│ Timeline Toolbar                                       │
│ [▶️ Play] [⏸️ Pause] [⏹️ Stop] [🏠 Start] [⏭️ End]     │
│ [➕ Add] [✂️ Cut] [🗑️ Del] [↩️ Undo] [↪️ Redo]         │
│ Zoom: [−] ═══●══════ [+]  View: 50ms/px              │
├────────────────────────────────────────────────────────┤
│ Timeline Canvas (HTML5)                                │
│ ┌──────────────────────────────────────────────────┐  │
│ │ 0s    1s    2s    3s    4s    5s    6s    7s    │  │
│ │ ┌──┐     ┌────┐        ┌──┐                     │  │
│ │ │▓▓│     │▓▓▓▓│        │▓▓│   Playhead           │  │
│ │ └──┘     └────┘        └──┘      ▼               │  │
│ │    ┌──┐           ┌──┐     │                     │  │
│ │    │▓▓│           │▓▓│     │                     │  │
│ │    └──┘           └──┘     │                     │  │
│ │                              │                     │  │
│ └──────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────┤
│ Status Bar: 47 events | 02:05.340 | 3 selected       │
└────────────────────────────────────────────────────────┘
```

### 5.3 Component Specifications

#### 5.3.1 Slot Component
- **Size:** 80px × 80px
- **Border:** 2px solid (normal), 3px (selected), dashed (empty)
- **Label:** Truncate with ellipsis at 12 chars
- **Shortcut Badge:** Small pill di top-right corner
- **Gain Indicator:** Subtle bar di bottom (0-200% range)

#### 5.3.2 Timeline Event Block
- **Height:** 40px
- **Min Width:** 20px
- **Border Radius:** 4px
- **Label:** Center-aligned, truncate if too short
- **Overlap Badge:** Circle dengan number, top-right

#### 5.3.3 Buttons
- **Size:** 32px × 32px (icon buttons), 100px × 32px (text buttons)
- **Hover:** Lighten background 10%
- **Active:** Darken background 10%
- **Disabled:** 50% opacity

### 5.4 Responsive Behavior

**Minimum Window Size:** 1024px × 600px
- SFX Board: Scroll if needed
- Timeline: Always fill available height
- Status bar: Always visible

**Panels Resizable:**
- SFX Board vs Timeline: Draggable divider (min 200px each)

---

## 6. Data Models

### 6.1 Slot Data Model

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Slot {
    pub id: String,              // UUID v4
    pub label: String,           // Max 32 chars
    pub audio_path: PathBuf,     // Absolute path
    pub shortcut: String,        // "Ctrl+Shift+A"
    pub gain: f32,               // 0.0 - 2.0
    pub duration_ms: f64,        // Auto-detected
    pub color: String,           // Hex color #RRGGBB
    pub created_at: DateTime<Utc>,
}
```

### 6.2 Timeline Event Model

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct TimelineEvent {
    pub event_id: String,        // UUID v4
    pub time_ms: f64,            // Timeline position
    pub slot_id: String,         // Reference to Slot
    pub audio_path: PathBuf,     // Denormalized for export
    pub label: String,           // Denormalized
    pub shortcut: String,        // Denormalized
    pub gain: f32,               // Copy dari slot
    pub duration_ms: f64,        // Copy dari slot
}
```

### 6.3 Project Model

```rust
#[derive(Serialize, Deserialize)]
pub struct Project {
    pub version: String,                    // "1.0.0"
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub settings: ProjectSettings,
    pub slots: Vec<Slot>,
    pub timeline: Timeline,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectSettings {
    pub global_shortcuts_enabled: bool,
    pub audio_buffer_size: u32,              // 512, 1024, 2048
    pub frame_rate: u32,                     // 30 fps
}

#[derive(Serialize, Deserialize)]
pub struct Timeline {
    pub events: Vec<TimelineEvent>,
    pub total_duration_ms: f64,
}
```

### 6.4 Recording Session Model

```rust
#[derive(Clone)]
pub struct RecordingSession {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub status: RecordingStatus,
    pub events_buffer: Vec<TimelineEvent>,
    pub current_time_ms: f64,
}

#[derive(Clone, PartialEq)]
pub enum RecordingStatus {
    Idle,
    Recording,
    Paused,
    Stopped,
}
```

---

## 7. Tauri Commands (Backend API)

### 7.1 Slot Management

```rust
#[tauri::command]
async fn add_slot(
    file_path: String,
    label: Option<String>,
) -> Result<Slot, String>

#[tauri::command]
async fn update_slot(
    slot_id: String,
    label: Option<String>,
    shortcut: Option<String>,
    gain: Option<f32>,
) -> Result<Slot, String>

#[tauri::command]
async fn delete_slot(slot_id: String) -> Result<(), String>

#[tauri::command]
async fn get_all_slots() -> Result<Vec<Slot>, String>

#[tauri::command]
async fn trigger_slot(slot_id: String) -> Result<(), String>
```

### 7.2 Recording

```rust
#[tauri::command]
async fn start_recording() -> Result<(), String>

#[tauri::command]
async fn pause_recording() -> Result<(), String>

#[tauri::command]
async fn resume_recording() -> Result<(), String>

#[tauri::command]
async fn stop_recording() -> Result<Vec<TimelineEvent>, String>

#[tauri::command]
async fn get_recording_status() -> Result<RecordingStatus, String>
```

### 7.3 Timeline

```rust
#[tauri::command]
async fn get_timeline_events() -> Result<Vec<TimelineEvent>, String>

#[tauri::command]
async fn add_timeline_event(
    slot_id: String,
    time_ms: f64,
) -> Result<TimelineEvent, String>

#[tauri::command]
async fn update_event_time(
    event_id: String,
    new_time_ms: f64,
) -> Result<(), String>

#[tauri::command]
async fn delete_timeline_events(
    event_ids: Vec<String>,
) -> Result<(), String>

#[tauri::command]
async fn duplicate_events(
    event_ids: Vec<String>,
) -> Result<Vec<TimelineEvent>, String>
```

### 7.4 Timeline Playback

```rust
#[tauri::command]
async fn play_timeline() -> Result<(), String>

#[tauri::command]
async fn pause_timeline() -> Result<(), String>

#[tauri::command]
async fn stop_timeline() -> Result<(), String>

#[tauri::command]
async fn seek_timeline(time_ms: f64) -> Result<(), String>

#[tauri::command]
async fn get_playback_position() -> Result<f64, String>
```

### 7.5 Project Management

```rust
#[tauri::command]
async fn save_project(file_path: String) -> Result<(), String>

#[tauri::command]
async fn load_project(file_path: String) -> Result<Project, String>

#[tauri::command]
async fn validate_audio_paths() -> Result<Vec<String>, String>

#[tauri::command]
async fn update_audio_path(
    old_path: String,
    new_path: String,
) -> Result<(), String>
```

### 7.6 Export

```rust
#[tauri::command]
async fn export_audio_wav(
    output_path: String,
) -> Result<(), String>

#[tauri::command]
async fn export_audio_mp3(
    output_path: String,
) -> Result<(), String>

#[tauri::command]
async fn export_timeline_json(
    output_path: String,
) -> Result<(), String>
```

### 7.7 Settings

```rust
#[tauri::command]
async fn set_global_shortcuts_enabled(
    enabled: bool,
) -> Result<(), String>

#[tauri::command]
async fn get_settings() -> Result<ProjectSettings, String>

#[tauri::command]
async fn update_settings(
    settings: ProjectSettings,
) -> Result<(), String>
```

---

## 8. Frontend Events (IPC from Backend)

```javascript
// Recording events
listen('recording-started', () => { /* update UI */ })
listen('recording-paused', () => { /* update UI */ })
listen('recording-resumed', () => { /* update UI */ })
listen('recording-stopped', (events) => { /* add to timeline */ })
listen('recording-time-update', ({ timeMs }) => { /* update timer */ })

// Playback events
listen('playback-started', () => { /* update UI */ })
listen('playback-paused', () => { /* update UI */ })
listen('playback-stopped', () => { /* reset playhead */ })
listen('playhead-update', ({ timeMs }) => { /* move playhead */ })
listen('event-triggered', ({ eventId }) => { /* visual flash */ })

// Slot events
listen('slot-triggered', ({ slotId }) => { /* pulse effect */ })

// Global shortcuts
listen('global-shortcut-triggered', ({ shortcut }) => { /* toast */ })

// Errors/warnings
listen('error', ({ message }) => { /* show error toast */ })
listen('warning', ({ message }) => { /* show warning toast */ })

// Export progress
listen('export-progress', ({ current, total }) => { /* update bar */ })
listen('export-complete', ({ files }) => { /* show success */ })
```

---

## 9. Technical Constraints & Requirements

### 9.1 Performance Requirements
- **Audio latency:** <50ms trigger-to-sound
- **Timeline rendering:** 60 FPS during playback
- **Project load time:** <2s for 500 events
- **Export speed:** Real-time or faster (1min audio = <1min export)
- **Memory usage:** <500MB untuk typical session (100 events, 50 slots)

### 9.2 Audio Requirements
- **Supported formats:** WAV (PCM), MP3 (via LAME decoder)
- **Sample rates:** 44.1kHz, 48kHz auto-detected
- **Bit depths:** 16-bit, 24-bit auto-detected
- **Channels:** Mono, Stereo (auto-mixed to stereo output)
- **Max audio file size:** 100MB per file
- **Max simultaneous playback:** 32 instances

### 9.3 File System Requirements
- **Project file size:** Typically <1MB (JSON only)
- **Autosave interval:** 2 minutes
- **Temp file cleanup:** On app exit
- **File permissions:** Read/write access to user's Documents folder

### 9.4 Windows Platform Requirements
- **OS Version:** Windows 10 (1809+) atau Windows 11
- **Architecture:** x86_64 only
- **Dependencies:** VC++ Redistributable (bundled)
- **Installer:** Portable executable (no installation required)

---

## 10. Error Handling & Edge Cases

### 10.1 Audio File Errors
- **File not found:** Show warning, disable slot, allow re-locate
- **Unsupported format:** Reject with clear message
- **Corrupted file:** Catch decode error, show warning
- **Permission denied:** Show error, suggest copying to accessible location

### 10.2 Recording Errors
- **Audio device unavailable:** Show error, disable recording
- **Disk full during autosave:** Warn user, disable autosave
- **Too many events (>10,000):** Warn performance degradation

### 10.3 Timeline Errors
- **Event time <0ms:** Clamp to 0ms
- **Event time >24 hours:** Warn user (unrealistic)
- **Overlapping manual edits:** Allow, show overlap indicator

### 10.4 Export Errors
- **Missing audio files:** Offer partial export atau cancel
- **Disk full:** Show error before starting export
- **Permission denied:** Show error, suggest different location

### 10.5 Global Shortcut Conflicts
- **Shortcut already used by system:** Detect and warn during assignment
- **Shortcut used by another slot:** Warn, prevent assignment
- **Global shortcuts disabled:** Show clear UI indicator

---

## 11. Future Enhancements (Out of Scope for v1.0)

### 11.1 Planned Features (v2.0+)
- **Waveform visualization** in timeline
- **Drag-and-drop** timeline editing
- **Markers/regions** for organizing timeline sections
- **MIDI controller support** untuk hardware triggering
- **VST plugin support** untuk effects processing
- **Project templates** preset packs
- **Collaboration** via cloud sync
- **Video preview** integration
- **Batch export** multiple formats simultaneously

### 11.2 Requested Features (Under Consideration)
- **Undo for recording sessions**
- **Multiple timelines** (layers)
- **Audio effects** (EQ, compression, reverb)
- **Sample library browser**
- **Scripting/automation** API
- **macOS dan Linux** ports

---

## 12. Testing Strategy

### 12.1 Unit Tests (Rust)
- Audio decoding/encoding
- Timeline event calculations
- Project serialization/deserialization
- Shortcut parsing and validation

### 12.2 Integration Tests
- Recording → Timeline → Export workflow
- Project save → Load consistency
- Multi-instance audio playback
- Global shortcut capture

### 12.3 Manual Testing Scenarios
1. Record 100 events, verify timeline accuracy
2. Export WAV/MP3, validate audio quality
3. Load project dengan missing files, verify warnings
4. Trigger 10 slots simultaneously, check mixing
5. Undo/redo 50 operations, verify state consistency
6. Zoom timeline 10 levels, check rendering performance
7. Global shortcuts while app minimized

### 12.4 Performance Testing
- Load project dengan 1000 events
- Export 10-minute timeline
- Playback timeline dengan 100 overlapping events
- Memory leak testing (8-hour session)

---

## 13. Documentation Requirements

### 13.1 User Documentation
- **Quick Start Guide:** 5-minute tutorial
- **Keyboard Shortcuts Reference:** In-app dan PDF
- **Export Format Specs:** JSON schema documentation
- **Troubleshooting:** Common issues dan solutions

### 13.2 Developer Documentation
- **Build Instructions:** Setup dev environment
- **Architecture Overview:** Component diagram
- **API Reference:** Tauri commands dan events
- **Contributing Guide:** Code style, PR process

---

## 14. Success Metrics & KPIs

### 14.1 User Adoption Metrics
- Downloads count
- Active users (30-day retention)
- Average session duration
- Projects created per user

### 14.2 Performance Metrics
- Average export time per minute of audio
- Timeline playback accuracy (±ms deviation)
- Crash rate (<0.1% sessions)
- Audio latency measurements

### 14.3 User Satisfaction Metrics
- Feature usage frequency (which features are used most)
- Error rate (user-facing errors per session)
- Autosave recovery success rate
- Export success rate (>99%)

---

## 15. Release Plan

### 15.1 Alpha Release (Internal Testing)
- Core features: Record, Timeline, Export
- Limited to 25 slots
- Basic error handling
- No autosave

### 15.2 Beta Release (Public Testing)
- All 25 slots enabled
- Autosave dan recovery
- Global shortcuts
- Polished UI

### 15.3 v1.0 Release (Production)
- Full feature set as per PRD
- Comprehensive documentation
- Installer/portable executable
- Performance optimizations
- Bug fixes dari beta feedback

### 15.4 Post-Launch Support
- Critical bug fixes within 48 hours
- Monthly patch releases
- Quarterly feature updates
- Community feedback integration

---

## 16. Appendix

### 16.1 Glossary
- **Slot:** Audio clip assignment dengan shortcut mapping
- **Event:** Timestamped instance of audio trigger di timeline
- **Mixdown:** Process menggabungkan multiple audio events menjadi single file
- **Nudge:** Fine-tuning event position by frame increments
- **Frame:** 1/30 second unit (~33.33ms) untuk frame-based precision

### 16.2 References
- Tauri Documentation: https://tauri.app/
- Rodio Audio Library: https://docs.rs/rodio/
- LAME MP3 Encoder: https://lame.sourceforge.io/
- React Documentation: https://react.dev/

### 16.3 Version History
- **v2.0 (This Document):** Rust + Tauri rewrite
- **v1.0 (Original):** Python + PySide6 implementation

---

**Document Status:** Final Draft  
**Approved By:** [Your Name]  
**Date:** 2026-04-25
