import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFolderOpen, faForward } from "@fortawesome/free-solid-svg-icons";

interface MissingFilesDialogProps {
  files: string[];
  openState: boolean;
  locating: boolean;
  onLocate: () => Promise<void>;
  onContinue: () => void;
}

export function MissingFilesDialog({
  files,
  openState,
  locating,
  onLocate,
  onContinue,
}: MissingFilesDialogProps) {
  if (!openState) {
    return null;
  }

  return (
    <section className="dialog-backdrop">
      <div className="dialog-card">
        <h2>Missing Audio Files</h2>
        <p>The following audio files could not be found:</p>

        <ul className="missing-files-list">
          {files.map((path) => (
            <li key={path}>
              <code>{path}</code>
            </li>
          ))}
        </ul>

        <div className="dialog-actions">
          <button type="button" onClick={() => void onLocate()} disabled={locating}>
            <FontAwesomeIcon icon={faFolderOpen} />
            {locating ? "Locating..." : "Locate Files"}
          </button>
          <button type="button" onClick={onContinue} disabled={locating}>
            <FontAwesomeIcon icon={faForward} />
            Continue Anyway
          </button>
        </div>
      </div>
    </section>
  );
}
