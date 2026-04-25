import type { PlaybackStatus } from "../../types";

interface TimelineToolbarProps {
  zoom: number;
  playbackStatus: PlaybackStatus;
  canDelete: boolean;
  canUndo: boolean;
  canRedo: boolean;
  onZoomChange: (zoom: number) => void;
  onPlay: () => Promise<void>;
  onPause: () => Promise<void>;
  onStop: () => Promise<void>;
  onSeekToStart: () => Promise<void>;
  onSeekToEnd: () => Promise<void>;
  onDeleteSelected: () => Promise<void>;
  onUndo: () => Promise<void>;
  onRedo: () => Promise<void>;
}

export function TimelineToolbar({
  zoom,
  playbackStatus,
  canDelete,
  canUndo,
  canRedo,
  onZoomChange,
  onPlay,
  onPause,
  onStop,
  onSeekToStart,
  onSeekToEnd,
  onDeleteSelected,
  onUndo,
  onRedo,
}: TimelineToolbarProps) {
  return (
    <div className="timeline-toolbar">
      <div className="timeline-toolbar-left">
        <button type="button" onClick={() => void onPlay()} disabled={playbackStatus === "Playing"}>
          Play
        </button>
        <button type="button" onClick={() => void onPause()} disabled={playbackStatus !== "Playing"}>
          Pause
        </button>
        <button type="button" onClick={() => void onStop()} disabled={playbackStatus === "Stopped"}>
          Stop
        </button>
        <button type="button" onClick={() => void onSeekToStart()}>
          Start
        </button>
        <button type="button" onClick={() => void onSeekToEnd()}>
          End
        </button>
        <button type="button" onClick={() => void onDeleteSelected()} disabled={!canDelete}>
          Delete
        </button>
        <button type="button" onClick={() => void onUndo()} disabled={!canUndo}>
          Undo
        </button>
        <button type="button" onClick={() => void onRedo()} disabled={!canRedo}>
          Redo
        </button>
      </div>

      <div className="zoom-controls">
        <button type="button" onClick={() => onZoomChange(Math.max(5, zoom * 0.8))}>
          -
        </button>
        <input
          type="range"
          min={5}
          max={300}
          value={zoom}
          onChange={(event) => onZoomChange(Number(event.currentTarget.value))}
        />
        <button type="button" onClick={() => onZoomChange(Math.min(300, zoom * 1.2))}>
          +
        </button>
        <span>{Math.round(zoom)} ms/px</span>
      </div>
    </div>
  );
}
