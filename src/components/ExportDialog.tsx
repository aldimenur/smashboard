import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faBan, faFileExport, faFolderOpen } from "@fortawesome/free-solid-svg-icons";

import type { ToastType } from "./Toast";

interface ExportDialogProps {
  openState: boolean;
  projectName: string;
  onClose: () => void;
  onToast?: (message: string, type?: ToastType) => void;
}

function sanitizePrefix(input: string): string {
  const trimmed = input.trim();
  if (!trimmed) {
    return "MyProject_";
  }

  return `${trimmed.replace(/[<>:"/\\|?*\x00-\x1F]/g, "_")}_`;
}

function joinPath(folder: string, fileName: string): string {
  if (folder.endsWith("\\") || folder.endsWith("/")) {
    return `${folder}${fileName}`;
  }

  return `${folder}\\${fileName}`;
}

function parsePathSelection(value: string | string[] | null): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value;
}

export function ExportDialog({ openState, projectName, onClose, onToast }: ExportDialogProps) {
  const [exportWav, setExportWav] = useState(true);
  const [exportMp3, setExportMp3] = useState(true);
  const [exportJson, setExportJson] = useState(true);
  const [outputFolder, setOutputFolder] = useState("");
  const [prefix, setPrefix] = useState("MyProject_");
  const [exporting, setExporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!openState) {
      return;
    }

    setPrefix(sanitizePrefix(projectName));
    setProgress(0);
    setError(null);
  }, [openState, projectName]);

  const selectedTargets = useMemo(() => {
    const targets: Array<"wav" | "mp3" | "json"> = [];
    if (exportWav) {
      targets.push("wav");
    }
    if (exportMp3) {
      targets.push("mp3");
    }
    if (exportJson) {
      targets.push("json");
    }
    return targets;
  }, [exportJson, exportMp3, exportWav]);

  if (!openState) {
    return null;
  }

  const browseFolder = async () => {
    const selected = parsePathSelection(
      await open({
        directory: true,
        multiple: false,
      }),
    );

    if (selected) {
      setOutputFolder(selected);
    }
  };

  const handleExport = async () => {
    if (selectedTargets.length === 0) {
      setError("Select at least one export format.");
      onToast?.("Select at least one export format", "error");
      return;
    }

    if (!outputFolder) {
      setError("Select an output folder.");
      onToast?.("Select an output folder", "error");
      return;
    }

    const missing = await invoke<string[]>("validate_audio_paths");
    let allowMissingFiles = false;

    if (missing.length > 0) {
      allowMissingFiles = await confirm(
        `${missing.length} audio file(s) are missing.\nContinue with partial export (skip missing files)?`,
        {
          title: "Export Validation",
          kind: "warning",
          okLabel: "Continue",
          cancelLabel: "Cancel",
        },
      );

      if (!allowMissingFiles) {
        setError("Export cancelled due to missing files.");
        onToast?.("Export cancelled due to missing files", "error");
        return;
      }
    }

    setExporting(true);
    setProgress(0);
    setError(null);

    try {
      let completed = 0;
      const safePrefix = prefix.trim() || "MyProject_";

      for (const target of selectedTargets) {
        if (target === "wav") {
          await invoke("export_audio_wav", {
            outputPath: joinPath(outputFolder, `${safePrefix}mixdown.wav`),
            allowMissingFiles,
          });
        } else if (target === "mp3") {
          await invoke("export_audio_mp3", {
            outputPath: joinPath(outputFolder, `${safePrefix}mixdown.mp3`),
            allowMissingFiles,
          });
        } else {
          await invoke("export_timeline_json", {
            outputPath: joinPath(outputFolder, `${safePrefix}timeline.json`),
          });
        }

        completed += 1;
        setProgress(Math.round((completed / selectedTargets.length) * 100));
      }

      onToast?.(`Export complete! ${selectedTargets.length} file(s) created.`, "success");

      onClose();
    } catch (err) {
      const message = `Export failed: ${String(err)}`;
      setError(message);
      onToast?.(message, "error");
    } finally {
      setExporting(false);
    }
  };

  return (
    <section className="dialog-backdrop">
      <div className="dialog-card">
        <h2>Export Project</h2>

        <div className="export-group">
          <h3>Audio Export</h3>
          <label>
            <input type="checkbox" checked={exportWav} onChange={(event) => setExportWav(event.currentTarget.checked)} />
            WAV (lossless, 44.1kHz/16-bit)
          </label>
          <label>
            <input type="checkbox" checked={exportMp3} onChange={(event) => setExportMp3(event.currentTarget.checked)} />
            MP3 (320kbps)
          </label>
        </div>

        <div className="export-group">
          <h3>Data Export</h3>
          <label>
            <input
              type="checkbox"
              checked={exportJson}
              onChange={(event) => setExportJson(event.currentTarget.checked)}
            />
            JSON (timeline data)
          </label>
        </div>

        <div className="export-group">
          <h3>Output Folder</h3>
          <div className="row">
            <input value={outputFolder} readOnly placeholder="Select output folder..." />
            <button type="button" onClick={() => void browseFolder()} disabled={exporting}>
              <FontAwesomeIcon icon={faFolderOpen} />
              Browse
            </button>
          </div>
        </div>

        <div className="export-group">
          <h3>Filename Prefix</h3>
          <input value={prefix} onChange={(event) => setPrefix(event.currentTarget.value)} disabled={exporting} />
        </div>

        {exporting ? (
          <div className="export-progress">
            <progress value={progress} max={100} />
            <span>Exporting... {progress}%</span>
          </div>
        ) : null}

        {error ? <p className="slot-error">{error}</p> : null}

        <div className="dialog-actions">
          <button type="button" onClick={onClose} disabled={exporting}>
            <FontAwesomeIcon icon={faBan} />
            Cancel
          </button>
          <button type="button" onClick={() => void handleExport()} disabled={exporting}>
            <FontAwesomeIcon icon={faFileExport} />
            Export
          </button>
        </div>
      </div>
    </section>
  );
}
