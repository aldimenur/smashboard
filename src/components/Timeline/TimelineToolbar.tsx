import type { PlaybackStatus } from "../../types";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faBackwardFast,
  faCirclePause,
  faCirclePlay,
  faCircleStop,
  faForwardFast,
  faRotateLeft,
  faRotateRight,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";

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
          <FontAwesomeIcon icon={faCirclePlay} />
          Play
        </button>
        <button type="button" onClick={() => void onPause()} disabled={playbackStatus !== "Playing"}>
          <FontAwesomeIcon icon={faCirclePause} />
          Pause
        </button>
        <button type="button" onClick={() => void onStop()} disabled={playbackStatus === "Stopped"}>
          <FontAwesomeIcon icon={faCircleStop} />
          Stop
        </button>
        <button type="button" onClick={() => void onSeekToStart()}>
          <FontAwesomeIcon icon={faBackwardFast} />
          Start
        </button>
        <button type="button" onClick={() => void onSeekToEnd()}>
          <FontAwesomeIcon icon={faForwardFast} />
          End
        </button>
        <button type="button" onClick={() => void onDeleteSelected()} disabled={!canDelete}>
          <FontAwesomeIcon icon={faTrash} />
          Delete
        </button>
        <button type="button" onClick={() => void onUndo()} disabled={!canUndo}>
          <FontAwesomeIcon icon={faRotateLeft} />
          Undo
        </button>
        <button type="button" onClick={() => void onRedo()} disabled={!canRedo}>
          <FontAwesomeIcon icon={faRotateRight} />
          Redo
        </button>
      </div>

      <div className="zoom-controls">
        <input
          type="range"
          min={5}
          max={300}
          value={zoom}
          onChange={(event) => onZoomChange(Number(event.currentTarget.value))}
          className="zoom-slider"
          aria-label="Timeline zoom"
        />
      </div>
    </div>
  );
}
