interface StatusBarProps {
  eventCount: number;
  durationMs: number;
  selectedCount: number;
  zoomMsPerPx: number;
}

function formatTime(ms: number): string {
  const totalMs = Math.max(0, Math.floor(ms));
  const minutes = Math.floor(totalMs / 60_000)
    .toString()
    .padStart(2, "0");
  const seconds = Math.floor((totalMs % 60_000) / 1000)
    .toString()
    .padStart(2, "0");
  const millis = (totalMs % 1000).toString().padStart(3, "0");
  return `${minutes}:${seconds}.${millis}`;
}

export function StatusBar({ eventCount, durationMs, selectedCount, zoomMsPerPx }: StatusBarProps) {
  return (
    <footer className="status-bar">
      <span>{eventCount} events</span>
      <span>|</span>
      <span>{formatTime(durationMs)}</span>
      <span>|</span>
      <span>{Math.round(zoomMsPerPx)} ms/px</span>
      {selectedCount > 0 ? (
        <>
          <span>|</span>
          <span>{selectedCount} selected</span>
        </>
      ) : null}
    </footer>
  );
}
