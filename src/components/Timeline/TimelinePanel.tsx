import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { StatusBar } from "../StatusBar";
import type { ToastType } from "../Toast";
import type { PlaybackStatus, TimelineEvent, UndoRedoState } from "../../types";
import { TimelineCanvas } from "./TimelineCanvas";
import { TimelineToolbar } from "./TimelineToolbar";

interface TimelinePanelProps {
  onToast?: (message: string, type?: ToastType) => void;
}

export function TimelinePanel({ onToast }: TimelinePanelProps) {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [playheadMs, setPlayheadMs] = useState(0);
  const [zoom, setZoom] = useState(50);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [playbackStatus, setPlaybackStatus] = useState<PlaybackStatus>("Stopped");
  const [undoRedoState, setUndoRedoState] = useState<UndoRedoState>({ canUndo: false, canRedo: false });
  const [error, setError] = useState<string | null>(null);

  const showToast = useCallback(
    (message: string, type: ToastType = "info") => {
      onToast?.(message, type);
    },
    [onToast],
  );

  const loadEvents = useCallback(async () => {
    try {
      const nextEvents = await invoke<TimelineEvent[]>("get_timeline_events");
      setEvents(nextEvents);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const loadUndoRedoState = useCallback(async () => {
    try {
      const nextState = await invoke<UndoRedoState>("get_undo_redo_state");
      setUndoRedoState(nextState);
    } catch {
      // Keep previous state when not available.
    }
  }, []);

  useEffect(() => {
    let unlistenPlayhead: UnlistenFn | undefined;
    let unlistenTimelineUpdated: UnlistenFn | undefined;
    let unlistenPlaybackStatus: UnlistenFn | undefined;

    void loadEvents();
    void loadUndoRedoState();

    void invoke<PlaybackStatus>("get_playback_status")
      .then((status) => {
        setPlaybackStatus(status);
      })
      .catch((err) => {
        setError(String(err));
      });

    void listen<number>("playhead-update", (event) => {
      setPlayheadMs(event.payload);
    }).then((fn) => {
      unlistenPlayhead = fn;
    });

    void listen("timeline-updated", () => {
      void loadEvents();
      void loadUndoRedoState();
    }).then((fn) => {
      unlistenTimelineUpdated = fn;
    });

    void listen<PlaybackStatus>("playback-status-updated", (event) => {
      setPlaybackStatus(event.payload);
    }).then((fn) => {
      unlistenPlaybackStatus = fn;
    });

    return () => {
      unlistenPlayhead?.();
      unlistenTimelineUpdated?.();
      unlistenPlaybackStatus?.();
    };
  }, [loadEvents, loadUndoRedoState]);

  const applyEventsCommit = useCallback(
    async (changes: Array<{ eventId: string; newTimeMs: number }>) => {
      if (changes.length === 0) {
        return;
      }

      try {
        await invoke("update_event_times", {
          updates: changes,
        });
        setError(null);
      } catch (err) {
        setError(String(err));
        await loadEvents();
      }
    },
    [loadEvents],
  );

  const commitPlayhead = useCallback(async (nextTimeMs: number) => {
    try {
      await invoke("seek_timeline", { timeMs: nextTimeMs });
      setPlayheadMs(nextTimeMs);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const deleteSelected = useCallback(async () => {
    if (selectedIds.length === 0) {
      return;
    }

    try {
      await invoke("delete_timeline_events", { eventIds: selectedIds });
      setSelectedIds([]);
      setError(null);
      showToast("Events deleted", "success");
    } catch (err) {
      setError(String(err));
      showToast(`Failed to delete events: ${String(err)}`, "error");
    }
  }, [selectedIds, showToast]);

  const seekToStart = useCallback(async () => {
    try {
      await invoke("seek_timeline", { timeMs: 0 });
      setPlayheadMs(0);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const seekToEnd = useCallback(async () => {
    const endMs =
      events.reduce((max, event) => Math.max(max, event.timeMs + event.durationMs), 0) + 1000;

    try {
      await invoke("seek_timeline", { timeMs: endMs });
      setPlayheadMs(endMs);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, [events]);

  const timelineDurationMs = useMemo(
    () => events.reduce((max, event) => Math.max(max, event.timeMs + event.durationMs), 0),
    [events],
  );

  const play = useCallback(async () => {
    try {
      await invoke("play_timeline");
      setPlaybackStatus("Playing");
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const pause = useCallback(async () => {
    try {
      await invoke("pause_timeline");
      setPlaybackStatus("Paused");
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const stop = useCallback(async () => {
    try {
      await invoke("stop_timeline");
      setPlaybackStatus("Stopped");
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const undo = useCallback(async () => {
    try {
      await invoke("undo");
      setError(null);
      showToast("Undo", "info");
    } catch (err) {
      setError(String(err));
      showToast(`Failed to undo: ${String(err)}`, "error");
    }
  }, [showToast]);

  const redo = useCallback(async () => {
    try {
      await invoke("redo");
      setError(null);
      showToast("Redo", "info");
    } catch (err) {
      setError(String(err));
      showToast(`Failed to redo: ${String(err)}`, "error");
    }
  }, [showToast]);

  return (
    <section className="timeline-panel">
      <TimelineToolbar
        zoom={zoom}
        playbackStatus={playbackStatus}
        canDelete={selectedIds.length > 0}
        canUndo={undoRedoState.canUndo}
        canRedo={undoRedoState.canRedo}
        onZoomChange={setZoom}
        onPlay={play}
        onPause={pause}
        onStop={stop}
        onSeekToStart={seekToStart}
        onSeekToEnd={seekToEnd}
        onDeleteSelected={deleteSelected}
        onUndo={undo}
        onRedo={redo}
      />

      {error ? <p className="slot-error">{error}</p> : null}

      <TimelineCanvas
        events={events}
        playheadMs={playheadMs}
        zoom={zoom}
        selectedIds={selectedIds}
        setSelectedIds={setSelectedIds}
        onPlayheadPreview={setPlayheadMs}
        onPlayheadCommit={commitPlayhead}
        onEventsPreview={setEvents}
        onEventsCommit={applyEventsCommit}
      />

      <StatusBar
        eventCount={events.length}
        durationMs={timelineDurationMs}
        selectedCount={selectedIds.length}
        zoomMsPerPx={zoom}
      />
    </section>
  );
}
