export interface Slot {
  id: string;
  position: number;
  label: string;
  audioPath: string;
  shortcut: string;
  gain: number;
  durationMs: number;
  color: string;
  createdAt: string;
}

export type RecordingStatus = "Idle" | "Recording" | "Paused" | "Stopped";

export interface TimelineEvent {
  eventId: string;
  timeMs: number;
  slotId: string;
  audioPath: string;
  label: string;
  shortcut: string;
  gain: number;
  durationMs: number;
}

export type PlaybackStatus = "Stopped" | "Playing" | "Paused";

export interface RecordingTimeUpdate {
  timeMs: number;
}

export interface ProjectSettings {
  globalShortcutsEnabled: boolean;
  audioBufferSize: number;
  frameRate: number;
  boardRows: number;
  boardColumns: number;
  boardLabel: string;
}

export interface TimelineData {
  events: TimelineEvent[];
  totalDurationMs: number;
}

export interface Project {
  version: string;
  projectName: string;
  createdAt: string;
  modifiedAt: string;
  settings: ProjectSettings;
  slots: Slot[];
  timeline: TimelineData;
}

export interface ProjectStatePayload {
  projectName: string;
  currentPath: string | null;
  hasUnsavedChanges: boolean;
  globalShortcutsEnabled: boolean;
  frameRate: number;
  boardRows: number;
  boardColumns: number;
  boardLabel: string;
}

export interface AutosaveRecoveryInfo {
  hasRecoverable: boolean;
  autosavePath: string;
  modifiedAt: string | null;
}

export interface UndoRedoState {
  canUndo: boolean;
  canRedo: boolean;
}

export interface RemoteControlStatus {
  running: boolean;
  port: number | null;
  token: string | null;
  url: string | null;
}
