import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faClock, faListUl, faMousePointer } from "@fortawesome/free-solid-svg-icons";

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
  void zoomMsPerPx;
  return (
    <footer className="status-bar">
      <span className="status-pill">
        <FontAwesomeIcon icon={faListUl} />
        {eventCount}
      </span>
      <span className="status-pill">
        <FontAwesomeIcon icon={faClock} />
        {formatTime(durationMs)}
      </span>
      {selectedCount > 0 ? (
        <span className="status-pill">
          <FontAwesomeIcon icon={faMousePointer} />
          {selectedCount}
        </span>
      ) : null}
    </footer>
  );
}
