import { useMemo, useState } from "react";

import type { TimelineEvent } from "../../types";

interface CanvasEvent extends TimelineEvent {
  track: number;
}

interface EventDragState {
  kind: "events";
  startX: number;
  eventIds: string[];
  initialTimes: Record<string, number>;
}

interface PlayheadDragState {
  kind: "playhead";
}

type DragState = EventDragState | PlayheadDragState;

interface TimelineCanvasProps {
  events: TimelineEvent[];
  playheadMs: number;
  zoom: number;
  slotColors?: Record<string, string>;
  selectedIds: string[];
  setSelectedIds: (ids: string[]) => void;
  onPlayheadPreview: (timeMs: number) => void;
  onPlayheadCommit: (timeMs: number) => Promise<void>;
  onEventsPreview: (nextEvents: TimelineEvent[]) => void;
  onEventsCommit: (eventTimes: Array<{ eventId: string; newTimeMs: number }>) => Promise<void>;
}

function colorFromId(id: string): string {
  let hash = 0;
  for (let index = 0; index < id.length; index += 1) {
    hash = (hash << 5) - hash + id.charCodeAt(index);
    hash |= 0;
  }

  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 60%, 45%)`;
}

function assignTracks(events: TimelineEvent[]): CanvasEvent[] {
  const sorted = [...events].sort((a, b) => a.timeMs - b.timeMs);
  const tracks: Array<{ endTime: number }> = [];

  return sorted.map((event) => {
    const trackIndex = tracks.findIndex((track) => track.endTime <= event.timeMs);

    if (trackIndex >= 0) {
      tracks[trackIndex].endTime = event.timeMs + event.durationMs;
      return { ...event, track: trackIndex };
    }

    tracks.push({ endTime: event.timeMs + event.durationMs });
    return { ...event, track: tracks.length - 1 };
  });
}

function formatTimelineTime(timeMs: number): string {
  const totalMs = Math.max(0, Math.floor(timeMs));
  const minutes = Math.floor(totalMs / 60_000)
    .toString()
    .padStart(2, "0");
  const seconds = Math.floor((totalMs % 60_000) / 1_000)
    .toString()
    .padStart(2, "0");
  const millis = (totalMs % 1_000).toString().padStart(3, "0");
  return `${minutes}:${seconds}.${millis}`;
}

export function TimelineCanvas({
  events,
  playheadMs,
  zoom,
  slotColors = {},
  selectedIds,
  setSelectedIds,
  onPlayheadPreview,
  onPlayheadCommit,
  onEventsPreview,
  onEventsCommit,
}: TimelineCanvasProps) {
  const [dragState, setDragState] = useState<DragState | null>(null);

  const trackedEvents = useMemo(() => assignTracks(events), [events]);
  const trackHeight = 40;
  const trackSpacing = 8;
  const trackCount = Math.max(1, trackedEvents.reduce((max, event) => Math.max(max, event.track + 1), 1));

  const width = Math.max(
    1200,
    Math.ceil(
      (Math.max(
        playheadMs,
        ...events.map((event) => event.timeMs + event.durationMs),
        30_000,
      ) +
        2000) /
        zoom,
    ),
  );
  const height = Math.max(220, trackCount * (trackHeight + trackSpacing) + 20);
  const clampTimeFromX = (x: number) => Math.max(0, Math.min((x * zoom), width * zoom));
  const playheadX = playheadMs / zoom;
  const playheadLabel = formatTimelineTime(playheadMs);
  const playheadLabelWidth = 86;
  const playheadLabelX = Math.max(4, Math.min(width - playheadLabelWidth - 4, playheadX - playheadLabelWidth / 2));

  const handleMouseDown = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = event.currentTarget;
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    const clicked = trackedEvents.find((item) => {
      const eventX = item.timeMs / zoom;
      const eventY = item.track * (trackHeight + trackSpacing) + 10;
      const eventW = Math.max(8, item.durationMs / zoom);
      const eventH = trackHeight;

      return x >= eventX && x <= eventX + eventW && y >= eventY && y <= eventY + eventH;
    });

    if (!clicked) {
      setSelectedIds([]);
      const nextPlayhead = clampTimeFromX(x);
      onPlayheadPreview(nextPlayhead);
      setDragState({
        kind: "playhead",
      });
      return;
    }

    let nextSelectedIds = [clicked.eventId];

    if (event.ctrlKey || event.metaKey) {
      nextSelectedIds = selectedIds.includes(clicked.eventId)
        ? selectedIds.filter((id) => id !== clicked.eventId)
        : [...selectedIds, clicked.eventId];
      if (nextSelectedIds.length === 0) {
        nextSelectedIds = [clicked.eventId];
      }
    }

    setSelectedIds(nextSelectedIds);

    const initialTimes = Object.fromEntries(
      events
        .filter((item) => nextSelectedIds.includes(item.eventId))
        .map((item) => [item.eventId, item.timeMs]),
    );

    setDragState({
      kind: "events",
      startX: x,
      eventIds: nextSelectedIds,
      initialTimes,
    });
  };

  const handleMouseMove = (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!dragState) {
      return;
    }

    const canvas = event.currentTarget;
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;

    if (dragState.kind === "playhead") {
      onPlayheadPreview(clampTimeFromX(x));
      return;
    }

    const deltaMs = (x - dragState.startX) * zoom;

    const nextEvents = events.map((item) => {
      const initialTime = dragState.initialTimes[item.eventId];
      if (initialTime === undefined) {
        return item;
      }

      return {
        ...item,
        timeMs: Math.max(0, initialTime + deltaMs),
      };
    });

    onEventsPreview(nextEvents);
  };

  const handleMouseUp = (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!dragState) {
      return;
    }

    if (dragState.kind === "playhead") {
      const canvas = event.currentTarget;
      const rect = canvas.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const nextPlayhead = clampTimeFromX(x);
      onPlayheadPreview(nextPlayhead);
      void onPlayheadCommit(nextPlayhead);
      setDragState(null);
      return;
    }

    const changed = events
      .filter((item) => dragState.eventIds.includes(item.eventId))
      .map((item) => ({
        eventId: item.eventId,
        newTimeMs: item.timeMs,
      }));

    void onEventsCommit(changed);
    setDragState(null);
  };

  return (
    <div className="timeline-canvas-wrap">
      <canvas
        className="timeline-canvas"
        width={width}
        height={height}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />

      <svg className="timeline-overlay" width={width} height={height}>
        <rect x={0} y={0} width={width} height={height} fill="#1A1A1A" />

        {Array.from({ length: Math.ceil((width * zoom) / 1000) + 1 }, (_, index) => {
          const x = (index * 1000) / zoom;
          return <line key={`grid-${index}`} x1={x} y1={0} x2={x} y2={height} stroke="#2A2A2A" strokeWidth={1} />;
        })}

        {trackedEvents.map((eventItem) => {
          const x = eventItem.timeMs / zoom;
          const y = eventItem.track * (trackHeight + trackSpacing) + 10;
          const widthPx = Math.max(8, eventItem.durationMs / zoom);
          const isSelected = selectedIds.includes(eventItem.eventId);
          const shortcut = eventItem.shortcut?.trim() || "--";
          const shortcutBadgeWidth = Math.max(18, Math.min(widthPx - 8, 58));
          const color = slotColors[eventItem.slotId] ?? colorFromId(eventItem.slotId);

          return (
            <g key={eventItem.eventId}>
              <rect
                x={x}
                y={y}
                width={widthPx}
                height={trackHeight}
                rx={6}
                fill={color}
                stroke={isSelected ? "#3B82F6" : "transparent"}
                strokeWidth={isSelected ? 2 : 0}
              />
              {widthPx >= 22 ? (
                <>
                  <rect x={x + 4} y={y + 7} width={shortcutBadgeWidth} height={18} rx={9} fill="rgba(0,0,0,0.35)" />
                  <text
                    x={x + 4 + shortcutBadgeWidth / 2}
                    y={y + 20}
                    fill="#FFFFFF"
                    fontSize={11}
                    textAnchor="middle"
                    fontWeight="700"
                  >
                    {shortcut}
                  </text>
                </>
              ) : null}
            </g>
          );
        })}

        <line
          x1={playheadX}
          y1={0}
          x2={playheadX}
          y2={height}
          stroke="#FF4D4D"
          strokeWidth={2}
        />
        <g>
          <rect x={playheadLabelX} y={4} width={playheadLabelWidth} height={18} rx={9} fill="#111827" stroke="#374151" />
          <text x={playheadLabelX + playheadLabelWidth / 2} y={17} fill="#F9FAFB" fontSize={11} textAnchor="middle">
            {playheadLabel}
          </text>
        </g>
      </svg>
    </div>
  );
}
