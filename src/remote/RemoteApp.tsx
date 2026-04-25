import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faMusic } from "@fortawesome/free-solid-svg-icons";
import { getSlotIcon } from "../utils/slotIcons";

type PlaybackStatus = "Stopped" | "Playing" | "Paused";
type RecordingStatus = "Idle" | "Recording" | "Paused";

interface Slot {
  id: string;
  position: number;
  label: string;
  shortcut: string;
  durationMs: number;
  color: string;
  audioPath: string;
  imageDataUrl?: string;
  iconName?: string;
}

interface RemoteState {
  projectName: string;
  boardRows: number;
  boardColumns: number;
  playheadMs: number;
  playbackStatus: PlaybackStatus;
  recordingStatus: RecordingStatus;
  slots: Slot[];
}

type TransportStatus = "connecting..." | "connected" | "reconnecting..." | "decode error";

const token = new URLSearchParams(window.location.search).get("token") ?? "";

function fmtMs(ms: number): string {
  const n = Math.max(0, Math.floor(ms || 0));
  const m = String(Math.floor(n / 60000)).padStart(2, "0");
  const s = String(Math.floor((n % 60000) / 1000)).padStart(2, "0");
  const mm = String(n % 1000).padStart(3, "0");
  return `${m}:${s}.${mm}`;
}

function toFileName(path: string, fallback: string): string {
  const parts = String(path || "")
    .split(/[\\/]/)
    .filter(Boolean);
  return parts[parts.length - 1] ?? fallback;
}

async function sendCommand(payload: { kind: string; slotId?: string }): Promise<void> {
  try {
    await fetch(`/api/command?token=${encodeURIComponent(token)}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(payload),
    });
  } catch {
    // Best effort for mobile UX.
  }
}

export function RemoteApp() {
  const [state, setState] = useState<RemoteState | null>(null);
  const [transportStatus, setTransportStatus] = useState<TransportStatus>("connecting...");
  const [pressed, setPressed] = useState<Record<string, boolean>>({});
  const reconnectTimerRef = useRef<number | null>(null);
  const sourceRef = useRef<EventSource | null>(null);

  const boardColumns = Math.max(1, state?.boardColumns ?? 1);
  const boardRows = Math.max(1, state?.boardRows ?? 1);
  const capacity = boardColumns * boardRows;

  const slotsByPosition = useMemo(() => {
    const map = new Map<number, Slot>();
    for (const slot of state?.slots ?? []) {
      map.set(slot.position, slot);
    }
    return map;
  }, [state?.slots]);

  const clearConnection = useCallback(() => {
    if (reconnectTimerRef.current !== null) {
      window.clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    if (sourceRef.current) {
      sourceRef.current.close();
      sourceRef.current = null;
    }
  }, []);

  const connect = useCallback(() => {
    clearConnection();
    setTransportStatus("connecting...");

    const source = new EventSource(`/api/events?token=${encodeURIComponent(token)}`);
    sourceRef.current = source;

    source.addEventListener("open", () => {
      setTransportStatus("connected");
    });

    source.addEventListener("state", (event) => {
      try {
        const payload = JSON.parse((event as MessageEvent).data) as RemoteState;
        setState(payload);
        setTransportStatus("connected");
      } catch {
        setTransportStatus("decode error");
      }
    });

    source.addEventListener("error", () => {
      setTransportStatus("reconnecting...");
      clearConnection();
      reconnectTimerRef.current = window.setTimeout(connect, 1200);
    });
  }, [clearConnection]);

  useEffect(() => {
    connect();
    return () => clearConnection();
  }, [clearConnection, connect]);

  const triggerSlot = useCallback(async (slotId: string) => {
    setPressed((prev) => ({ ...prev, [slotId]: true }));
    window.setTimeout(() => {
      setPressed((prev) => ({ ...prev, [slotId]: false }));
    }, 220);
    await sendCommand({ kind: "trigger_slot", slotId });
  }, []);

  return (
    <main className="remote-app">
      <header className="remote-top">
        <div>
          <h1 className="remote-project">{state?.projectName || "SFX Board Remote"}</h1>
          <p className="remote-meta">
            {transportStatus} |{" "}
            {state
              ? `${state.playbackStatus} | ${state.recordingStatus} | ${fmtMs(state.playheadMs)}`
              : "waiting for state..."}
          </p>
        </div>
        <button className="remote-btn" type="button" onClick={() => void sendCommand({ kind: "stop_all_audio" })}>
          Stop All
        </button>
      </header>

      <section className="remote-grid" style={{ gridTemplateColumns: `repeat(${boardColumns}, minmax(0, 1fr))` }}>
        {Array.from({ length: capacity }, (_, index) => {
          const slot = slotsByPosition.get(index);
          if (!slot) {
            return (
              <article key={`empty-${index}`} className="remote-slot remote-slot-empty">
                <strong className="remote-slot-label">Empty</strong>
                <div className="remote-slot-meta">Slot {index + 1}</div>
              </article>
            );
          }

          return (
            <article
              key={slot.id}
              className={`remote-slot remote-slot-loaded ${pressed[slot.id] ? "remote-slot-playing" : ""} ${
                slot.imageDataUrl ? "remote-slot-with-full-image" : ""
              }`}
              style={{ "--slot-color": slot.color || "#3a3a3a" } as CSSProperties}
              role="button"
              tabIndex={0}
              onClick={() => void triggerSlot(slot.id)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  void triggerSlot(slot.id);
                }
              }}
              aria-label={`Trigger slot ${slot.label}`}
            >
              {slot.imageDataUrl ? <img className="remote-slot-full-image" src={slot.imageDataUrl} alt="" /> : null}
              {slot.imageDataUrl ? (
                <span
                  className={`remote-slot-shortcut remote-slot-shortcut-floating ${
                    slot.shortcut ? "remote-slot-shortcut-active" : ""
                  }`}
                >
                  {slot.shortcut || "--"}
                </span>
              ) : null}
              <div className="remote-slot-details">
                <div className="remote-slot-content">
                  {!slot.imageDataUrl ? (
                    <div className="remote-slot-image remote-slot-image-empty" aria-hidden="true">
                      <FontAwesomeIcon icon={getSlotIcon(slot.iconName) ?? faMusic} />
                    </div>
                  ) : null}
                  <div className="remote-slot-content-text">
                    <div className="remote-slot-head">
                      <strong className="remote-slot-label" title={slot.label}>
                        {slot.label || "Untitled"}
                      </strong>
                      {!slot.imageDataUrl ? (
                        <span className={`remote-slot-shortcut ${slot.shortcut ? "remote-slot-shortcut-active" : ""}`}>
                          {slot.shortcut || "--"}
                        </span>
                      ) : null}
                    </div>
                    <div className="remote-slot-meta">
                      <span className="remote-slot-file" title={toFileName(slot.audioPath, slot.label)}>
                        {toFileName(slot.audioPath, slot.label)}
                      </span>
                      <span>{(slot.durationMs / 1000).toFixed(2)}s</span>
                    </div>
                  </div>
                </div>
              </div>
            </article>
          );
        })}
      </section>
    </main>
  );
}
