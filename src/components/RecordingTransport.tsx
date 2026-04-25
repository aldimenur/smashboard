import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCirclePause, faCirclePlay, faCircleStop, faCircle as faRecord } from "@fortawesome/free-solid-svg-icons";

import type { RecordingStatus, RecordingTimeUpdate, TimelineEvent } from "../types";

function formatTime(timeMs: number): string {
  const totalMs = Math.max(0, Math.floor(timeMs));
  const minutes = Math.floor(totalMs / 60_000)
    .toString()
    .padStart(2, "0");
  const seconds = Math.floor((totalMs % 60_000) / 1_000)
    .toString()
    .padStart(2, "0");
  const millis = Math.floor(totalMs % 1_000)
    .toString()
    .padStart(3, "0");

  return `${minutes}:${seconds}.${millis}`;
}

export function RecordingTransport() {
  const [status, setStatus] = useState<RecordingStatus>("Idle");
  const [timeMs, setTimeMs] = useState(0);
  const [eventCount, setEventCount] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const statusRef = useRef<RecordingStatus>("Idle");

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  useEffect(() => {
    let unlistenTime: UnlistenFn | undefined;
    let unlistenEvent: UnlistenFn | undefined;

    void invoke<RecordingStatus>("get_recording_status")
      .then((result) => {
        setStatus(result);
      })
      .catch((err) => {
        setError(String(err));
      });

    void listen<RecordingTimeUpdate>("recording-time-update", (event) => {
      setTimeMs(event.payload.timeMs);
    }).then((fn) => {
      unlistenTime = fn;
    });

    void listen<TimelineEvent>("recording-event-captured", () => {
      if (statusRef.current === "Recording") {
        setEventCount((prev) => prev + 1);
      }
    }).then((fn) => {
      unlistenEvent = fn;
    });

    return () => {
      unlistenTime?.();
      unlistenEvent?.();
    };
  }, []);

  const controls = useMemo(
    () => ({
      canRecord: status === "Idle" || status === "Stopped",
      canPause: status === "Recording",
      canResume: status === "Paused",
      canStop: status === "Recording" || status === "Paused",
    }),
    [status],
  );

  const handleRecord = async () => {
    try {
      await invoke("start_recording");
      setStatus("Recording");
      setTimeMs(0);
      setEventCount(0);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handlePause = async () => {
    try {
      await invoke("pause_recording");
      setStatus("Paused");
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleResume = async () => {
    try {
      await invoke("resume_recording");
      setStatus("Recording");
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleStop = async () => {
    try {
      const events = await invoke<TimelineEvent[]>("stop_recording");
      setStatus("Idle");
      setEventCount(events.length);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <section className="recording-transport">
      <div className="recording-buttons">
        <button type="button" onClick={() => void handleRecord()} disabled={!controls.canRecord}>
          <FontAwesomeIcon icon={faRecord} />
          Record
        </button>
        <button type="button" onClick={() => void handlePause()} disabled={!controls.canPause}>
          <FontAwesomeIcon icon={faCirclePause} />
          Pause
        </button>
        <button type="button" onClick={() => void handleResume()} disabled={!controls.canResume}>
          <FontAwesomeIcon icon={faCirclePlay} />
          Resume
        </button>
        <button type="button" onClick={() => void handleStop()} disabled={!controls.canStop}>
          <FontAwesomeIcon icon={faCircleStop} />
          Stop
        </button>
      </div>
      <div className="recording-status">
        <span>Status: {status}</span>
        <span>Time: {formatTime(timeMs)}</span>
        <span>Events: {eventCount}</span>
      </div>
      {error ? <p className="slot-error">{error}</p> : null}
    </section>
  );
}
