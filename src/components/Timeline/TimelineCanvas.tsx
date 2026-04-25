import { useMemo, useState } from "react";

import type { TimelineEvent } from "../../types";

interface CanvasEvent extends TimelineEvent {
  track: number;
}

interface DragState {
  startX: number;
  eventIds: string[];
  initialTimes: Record<string, number>;
}

type OverlapMap = Map<string, number>;

interface TimelineCanvasProps {
  events: TimelineEvent[];
  playheadMs: number;
  zoom: number;
  selectedIds: string[];
  setSelectedIds: (ids: string[]) => void;
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

function detectOverlaps(events: TimelineEvent[]): OverlapMap {
  const overlapCounts = new Map<string, number>();

  for (let indexA = 0; indexA < events.length; indexA += 1) {
    const eventA = events[indexA];
    const aStart = eventA.timeMs;
    const aEnd = eventA.timeMs + eventA.durationMs;
    let overlapCount = 0;

    for (let indexB = 0; indexB < events.length; indexB += 1) {
      if (indexA === indexB) {
        continue;
      }

      const eventB = events[indexB];
      const bStart = eventB.timeMs;
      const bEnd = eventB.timeMs + eventB.durationMs;

      if (aStart < bEnd && aEnd > bStart) {
        overlapCount += 1;
      }
    }

    if (overlapCount > 0) {
      overlapCounts.set(eventA.eventId, overlapCount);
    }
  }

  return overlapCounts;
}

export function TimelineCanvas({
  events,
  playheadMs,
  zoom,
  selectedIds,
  setSelectedIds,
  onEventsPreview,
  onEventsCommit,
}: TimelineCanvasProps) {
  const [dragState, setDragState] = useState<DragState | null>(null);

  const trackedEvents = useMemo(() => assignTracks(events), [events]);
  const overlapCounts = useMemo(() => detectOverlaps(events), [events]);
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
      setDragState(null);
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

  const handleMouseUp = () => {
    if (!dragState) {
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
          const overlapCount = overlapCounts.get(eventItem.eventId) ?? 0;
          const badgeX = x + widthPx - 10;
          const badgeY = y + 10;

          return (
            <g key={eventItem.eventId}>
              <rect
                x={x}
                y={y}
                width={widthPx}
                height={trackHeight}
                rx={6}
                fill={colorFromId(eventItem.slotId)}
                stroke={isSelected ? "#3B82F6" : "transparent"}
                strokeWidth={isSelected ? 2 : 0}
              />
              <text x={x + 6} y={y + 22} fill="#FFFFFF" fontSize={12}>
                {eventItem.label}
              </text>

              {overlapCount > 0 ? (
                <g>
                  <circle cx={badgeX} cy={badgeY} r={10} fill="#EF4444" />
                  <text x={badgeX} y={badgeY + 3} fill="#FFFFFF" fontSize={10} textAnchor="middle" fontWeight="700">
                    x{overlapCount}
                  </text>
                </g>
              ) : null}
            </g>
          );
        })}

        <line
          x1={playheadMs / zoom}
          y1={0}
          x2={playheadMs / zoom}
          y2={height}
          stroke="#FF4D4D"
          strokeWidth={2}
        />
      </svg>
    </div>
  );
}
