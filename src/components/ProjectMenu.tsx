import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import QRCode from "qrcode";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faArrowsRotate,
  faMobileScreenButton,
  faQrcode,
  faDownload,
  faFileArrowDown,
  faFileCirclePlus,
  faFileExport,
  faFileImport,
  faPowerOff,
} from "@fortawesome/free-solid-svg-icons";

import { MissingFilesDialog } from "./MissingFilesDialog";
import type { ToastType } from "./Toast";
import type { ProjectStatePayload, RemoteControlStatus } from "../types";

interface ProjectMenuProps {
  onOpenExport: () => void;
  onProjectNameChange: (projectName: string) => void;
  onToast?: (message: string, type?: ToastType) => void;
}

const EMPTY_PROJECT_STATE: ProjectStatePayload = {
  projectName: "Untitled",
  currentPath: null,
  hasUnsavedChanges: false,
  globalShortcutsEnabled: false,
  frameRate: 30,
  boardRows: 5,
  boardColumns: 5,
  boardLabel: "SFX Board",
};

function ensureProjectExtension(path: string): string {
  return path.toLowerCase().endsWith(".sfxproj") ? path : `${path}.sfxproj`;
}

function getFileName(path: string | null): string {
  if (!path) {
    return "Untitled.sfxproj";
  }

  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? "Untitled.sfxproj";
}

function formatAutosaveText(timestamp: string | null): string {
  if (!timestamp) {
    return "Autosave: waiting";
  }

  return `Autosave: ${new Date(timestamp).toLocaleTimeString()}`;
}

function parsePathSelection(value: string | string[] | null): string | null {
  if (Array.isArray(value)) {
    return value[0] ?? null;
  }
  return value;
}

export function ProjectMenu({
  onOpenExport,
  onProjectNameChange,
  onToast,
}: ProjectMenuProps) {
  const [projectState, setProjectState] = useState<ProjectStatePayload>(EMPTY_PROJECT_STATE);
  const [lastAutosaveAt, setLastAutosaveAt] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [remoteStatus, setRemoteStatus] = useState<RemoteControlStatus | null>(null);
  const [remoteModalOpen, setRemoteModalOpen] = useState(false);
  const [remoteQrDataUrl, setRemoteQrDataUrl] = useState<string | null>(null);
  const [missingFiles, setMissingFiles] = useState<string[]>([]);
  const [missingDialogOpen, setMissingDialogOpen] = useState(false);
  const [locatingMissingFiles, setLocatingMissingFiles] = useState(false);
  const projectStateRef = useRef(projectState);
  const allowImmediateCloseRef = useRef(false);

  useEffect(() => {
    projectStateRef.current = projectState;
  }, [projectState]);

  const showToast = useCallback(
    (message: string, type: ToastType = "info") => {
      onToast?.(message, type);
    },
    [onToast],
  );

  const refreshProjectState = useCallback(async () => {
    try {
      const nextState = await invoke<ProjectStatePayload>("get_project_state");
      setProjectState(nextState);
      onProjectNameChange(nextState.projectName);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, [onProjectNameChange]);

  const refreshRemoteStatus = useCallback(async () => {
    try {
      const status = await invoke<RemoteControlStatus>("get_remote_control_status");
      setRemoteStatus(status);
    } catch {
      setRemoteStatus(null);
    }
  }, []);

  const updateWindowTitle = useCallback(async (state: ProjectStatePayload) => {
    const fileName = getFileName(state.currentPath);
    const suffix = state.hasUnsavedChanges ? "*" : "";
    const title = `SFX Board - ${fileName}${suffix}`;

    try {
      await getCurrentWindow().setTitle(title);
    } catch {
      // Best-effort title update.
    }
  }, []);

  const resolveMissingAudioPaths = useCallback(async () => {
    const missing = await invoke<string[]>("validate_audio_paths");

    if (missing.length === 0) {
      return;
    }

    setMissingFiles(missing);
    setMissingDialogOpen(true);
    showToast(`Project loaded with ${missing.length} missing file(s)`, "error");
  }, [showToast]);

  const locateMissingFiles = useCallback(async () => {
    if (missingFiles.length === 0) {
      return;
    }

    setLocatingMissingFiles(true);

    try {
      for (const oldPath of missingFiles) {
        const selected = parsePathSelection(
          await open({
            multiple: false,
            directory: false,
            defaultPath: oldPath,
            filters: [{ name: "Audio", extensions: ["wav", "mp3"] }],
          }),
        );

        if (!selected) {
          continue;
        }

        await invoke("update_audio_path", {
          oldPath,
          newPath: selected,
        });
      }

      const remaining = await invoke<string[]>("validate_audio_paths");
      setMissingFiles(remaining);

      if (remaining.length === 0) {
        setMissingDialogOpen(false);
        showToast("All missing files resolved", "success");
      } else {
        showToast(`${remaining.length} missing file(s) still unresolved`, "error");
      }
    } catch (err) {
      setError(`Failed to locate files: ${String(err)}`);
      showToast(`Failed to locate files: ${String(err)}`, "error");
    } finally {
      setLocatingMissingFiles(false);
    }
  }, [missingFiles, showToast]);

  const saveProject = useCallback(
    async (forceDialog: boolean) => {
      if (busy) {
        return;
      }

      setBusy(true);
      try {
        let targetPath = projectState.currentPath;
        if (forceDialog || !targetPath) {
          const selectedPath = await save({
            title: "Save SFX Project",
            defaultPath: targetPath ?? `${projectState.projectName}.sfxproj`,
            filters: [{ name: "SFX Project", extensions: ["sfxproj"] }],
          });

          if (!selectedPath) {
            return;
          }

          targetPath = ensureProjectExtension(selectedPath);
        }

        await invoke("save_project", { filePath: targetPath });
        await refreshProjectState();
        setStatusMessage("Project saved");
        setError(null);
        showToast("Project saved", "success");
      } catch (err) {
        setError(String(err));
        showToast(`Failed to save project: ${String(err)}`, "error");
      } finally {
        setBusy(false);
      }
    },
    [busy, projectState.currentPath, projectState.projectName, refreshProjectState, showToast],
  );

  const requestCloseApp = useCallback(async () => {
    try {
      await getCurrentWindow().close();
    } catch {
      try {
        await invoke("force_quit_app");
      } catch (err) {
        setError(`Failed to close app: ${String(err)}`);
        showToast(`Failed to close app: ${String(err)}`, "error");
      }
    }
  }, [showToast]);

  const openProject = useCallback(async () => {
    if (busy) {
      return;
    }

    setBusy(true);
    try {
      if (projectState.hasUnsavedChanges) {
        const shouldDiscard = await confirm("Discard unsaved changes and open another project?", {
          title: "Unsaved Changes",
          kind: "warning",
          okLabel: "Discard",
          cancelLabel: "Cancel",
        });

        if (!shouldDiscard) {
          return;
        }
      }

      const selected = parsePathSelection(
        await open({
          multiple: false,
          directory: false,
          filters: [{ name: "SFX Project", extensions: ["sfxproj"] }],
        }),
      );

      if (!selected) {
        return;
      }

      await invoke("load_project", { filePath: selected });
      await resolveMissingAudioPaths();
      await refreshProjectState();
      setStatusMessage("Project loaded");
      setError(null);
      showToast("Project loaded", "success");
    } catch (err) {
      setError(`Failed to open project: ${String(err)}`);
      showToast(`Failed to open project: ${String(err)}`, "error");
    } finally {
      setBusy(false);
    }
  }, [busy, projectState.hasUnsavedChanges, refreshProjectState, resolveMissingAudioPaths, showToast]);

  const createNewProject = useCallback(async () => {
    if (busy) {
      return;
    }

    setBusy(true);
    try {
      if (projectState.hasUnsavedChanges) {
        const shouldDiscard = await confirm("Discard unsaved changes and create a new project?", {
          title: "Unsaved Changes",
          kind: "warning",
          okLabel: "Discard",
          cancelLabel: "Cancel",
        });

        if (!shouldDiscard) {
          return;
        }
      }

      await invoke("new_project");
      await refreshProjectState();
      setStatusMessage("New project created");
      setError(null);
      showToast("New project created", "success");
    } catch (err) {
      setError(`Failed to create new project: ${String(err)}`);
      showToast(`Failed to create new project: ${String(err)}`, "error");
    } finally {
      setBusy(false);
    }
  }, [busy, projectState.hasUnsavedChanges, refreshProjectState, showToast]);

  const resetTimeline = useCallback(async () => {
    if (busy) {
      return;
    }

    const shouldReset = await confirm("Reset timeline and remove all events?", {
      title: "Reset Timeline",
      kind: "warning",
      okLabel: "Reset",
      cancelLabel: "Cancel",
    });

    if (!shouldReset) {
      return;
    }

    setBusy(true);
    try {
      await invoke("reset_timeline");
      await refreshProjectState();
      setStatusMessage("Timeline reset");
      setError(null);
      showToast("Timeline reset", "success");
    } catch (err) {
      setError(`Failed to reset timeline: ${String(err)}`);
      showToast(`Failed to reset timeline: ${String(err)}`, "error");
    } finally {
      setBusy(false);
    }
  }, [busy, refreshProjectState, showToast]);

  useEffect(() => {
    void updateWindowTitle(projectState);
  }, [projectState, updateWindowTitle]);

  useEffect(() => {
    let unlistenTimelineUpdated: UnlistenFn | undefined;
    let unlistenProjectLoaded: UnlistenFn | undefined;
    let unlistenAutosave: UnlistenFn | undefined;

    void refreshProjectState();
    void refreshRemoteStatus();

    void listen("timeline-updated", () => {
      void refreshProjectState();
    }).then((fn) => {
      unlistenTimelineUpdated = fn;
    });

    void listen("project-loaded", () => {
      void refreshProjectState();
    }).then((fn) => {
      unlistenProjectLoaded = fn;
    });

    void listen<string>("autosave-completed", () => {
      setLastAutosaveAt(new Date().toISOString());
      void refreshProjectState();
    }).then((fn) => {
      unlistenAutosave = fn;
    });

    const interval = window.setInterval(() => {
      void refreshProjectState();
    }, 2000);

    return () => {
      window.clearInterval(interval);
      unlistenTimelineUpdated?.();
      unlistenProjectLoaded?.();
      unlistenAutosave?.();
    };
  }, [refreshProjectState, refreshRemoteStatus]);

  useEffect(() => {
    if (!remoteModalOpen || !remoteStatus?.running || !remoteStatus?.url) {
      setRemoteQrDataUrl(null);
      return;
    }

    void QRCode.toDataURL(remoteStatus.url, {
      margin: 1,
      width: 200,
      color: {
        dark: "#111827",
        light: "#f9fafb",
      },
    })
      .then((url: string) => {
        setRemoteQrDataUrl(url);
      })
      .catch(() => {
        setRemoteQrDataUrl(null);
      });
  }, [remoteModalOpen, remoteStatus?.running, remoteStatus?.url]);

  const toggleRemoteControl = useCallback(async () => {
    try {
      const current = await invoke<RemoteControlStatus>("get_remote_control_status");
      const next = current.running
        ? await invoke<RemoteControlStatus>("stop_remote_control")
        : await invoke<RemoteControlStatus>("start_remote_control", { port: 8765 });
      setRemoteStatus(next);
      showToast(next.running ? "Remote control enabled" : "Remote control disabled", "success");
    } catch (err) {
      setError(`Failed to toggle remote control: ${String(err)}`);
      showToast(`Failed to toggle remote control: ${String(err)}`, "error");
    }
  }, [showToast]);

  useEffect(() => {
    let unlistenCloseRequested: UnlistenFn | undefined;

    void getCurrentWindow()
      .onCloseRequested(async (event) => {
        if (allowImmediateCloseRef.current) {
          return;
        }

        event.preventDefault();

        if (projectStateRef.current.hasUnsavedChanges) {
          let shouldDiscard = false;
          try {
            shouldDiscard = await confirm("You have unsaved changes. Close anyway?", {
              title: "Unsaved Changes",
              kind: "warning",
              okLabel: "Close",
              cancelLabel: "Cancel",
            });
          } catch {
            return;
          }

          if (!shouldDiscard) {
            return;
          }
        }

        allowImmediateCloseRef.current = true;
        try {
          await invoke("force_quit_app");
        } catch (err) {
          allowImmediateCloseRef.current = false;
          setError(`Failed to close app: ${String(err)}`);
          showToast(`Failed to close app: ${String(err)}`, "error");
        }
      })
      .then((fn) => {
        unlistenCloseRequested = fn;
      });

    return () => {
      unlistenCloseRequested?.();
    };
  }, []);

  const saveLabel = useMemo(
    () => `Save${projectState.hasUnsavedChanges ? " *" : ""}`,
    [projectState.hasUnsavedChanges],
  );

  return (
    <section className="project-menu">
      <div className="project-menu-buttons">
        <button type="button" onClick={() => void saveProject(false)} disabled={busy}>
          <FontAwesomeIcon icon={faFileArrowDown} />
          {saveLabel}
        </button>
        <button type="button" onClick={() => void saveProject(true)} disabled={busy}>
          <FontAwesomeIcon icon={faDownload} />
          Save As
        </button>
        <button type="button" onClick={() => void openProject()} disabled={busy}>
          <FontAwesomeIcon icon={faFileImport} />
          Open
        </button>
        <button type="button" onClick={() => void createNewProject()} disabled={busy}>
          <FontAwesomeIcon icon={faFileCirclePlus} />
          New
        </button>
        <button type="button" onClick={onOpenExport} disabled={busy}>
          <FontAwesomeIcon icon={faFileExport} />
          Export
        </button>
        <button type="button" onClick={() => void resetTimeline()} disabled={busy}>
          <FontAwesomeIcon icon={faArrowsRotate} />
          Reset Timeline
        </button>
        <button type="button" onClick={() => setRemoteModalOpen(true)}>
          <FontAwesomeIcon icon={faMobileScreenButton} />
          Remote
        </button>
        <button type="button" onClick={() => void requestCloseApp()} className="button-danger-soft">
          <FontAwesomeIcon icon={faPowerOff} />
          Quit
        </button>
      </div>

      <div className="project-menu-meta">
        <span>{getFileName(projectState.currentPath)}</span>
        <span>{formatAutosaveText(lastAutosaveAt)}</span>
        {remoteStatus?.running && remoteStatus.url ? <span>Remote: {remoteStatus.url}</span> : null}
      </div>

      {statusMessage ? <p className="project-status">{statusMessage}</p> : null}
      {error ? <p className="slot-error">{error}</p> : null}

      {remoteModalOpen ? (
        <section
          className="dialog-backdrop"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) {
              setRemoteModalOpen(false);
            }
          }}
        >
          <div className="dialog-card remote-modal-card" onMouseDown={(event) => event.stopPropagation()}>
            <h2>
              <FontAwesomeIcon icon={faMobileScreenButton} />
              Remote Control
            </h2>
            <p className="slot-editor-hint">
              Status: {remoteStatus?.running ? "Running" : "Stopped"}
              {remoteStatus?.port ? ` (:${remoteStatus.port})` : ""}
            </p>
            <div className="dialog-actions">
              <button type="button" onClick={() => void toggleRemoteControl()}>
                <FontAwesomeIcon icon={faMobileScreenButton} />
                {remoteStatus?.running ? "Turn Off" : "Turn On"}
              </button>
              <button type="button" onClick={() => setRemoteModalOpen(false)}>
                Close
              </button>
            </div>

            {remoteStatus?.running && remoteStatus.url ? (
              <>
                <p className="slot-editor-hint">{remoteStatus.url}</p>
                <div className="remote-qr-panel">
                  <div className="remote-qr-title">
                    <FontAwesomeIcon icon={faQrcode} />
                    Scan to Connect
                  </div>
                  {remoteQrDataUrl ? (
                    <img src={remoteQrDataUrl} alt="Remote control QR code" />
                  ) : (
                    <span>Generating QR...</span>
                  )}
                </div>
              </>
            ) : null}
          </div>
        </section>
      ) : null}

      <MissingFilesDialog
        files={missingFiles}
        openState={missingDialogOpen}
        locating={locatingMissingFiles}
        onLocate={locateMissingFiles}
        onContinue={() => setMissingDialogOpen(false)}
      />
    </section>
  );
}
