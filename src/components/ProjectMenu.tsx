import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";

import { MissingFilesDialog } from "./MissingFilesDialog";
import type { ToastType } from "./Toast";
import type { ProjectStatePayload } from "../types";

interface ProjectMenuProps {
  onOpenExport: () => void;
  onOpenShortcuts: () => void;
  onProjectNameChange: (projectName: string) => void;
  onToast?: (message: string, type?: ToastType) => void;
}

const EMPTY_PROJECT_STATE: ProjectStatePayload = {
  projectName: "Untitled",
  currentPath: null,
  hasUnsavedChanges: false,
  globalShortcutsEnabled: false,
  frameRate: 30,
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
  onOpenShortcuts,
  onProjectNameChange,
  onToast,
}: ProjectMenuProps) {
  const [projectState, setProjectState] = useState<ProjectStatePayload>(EMPTY_PROJECT_STATE);
  const [lastAutosaveAt, setLastAutosaveAt] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
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

  useEffect(() => {
    void updateWindowTitle(projectState);
  }, [projectState, updateWindowTitle]);

  useEffect(() => {
    let unlistenTimelineUpdated: UnlistenFn | undefined;
    let unlistenProjectLoaded: UnlistenFn | undefined;
    let unlistenAutosave: UnlistenFn | undefined;

    void refreshProjectState();

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
  }, [refreshProjectState]);

  useEffect(() => {
    let unlistenCloseRequested: UnlistenFn | undefined;

    void getCurrentWindow()
      .onCloseRequested(async (event) => {
        if (allowImmediateCloseRef.current) {
          return;
        }

        if (!projectStateRef.current.hasUnsavedChanges) {
          return;
        }

        event.preventDefault();

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

        allowImmediateCloseRef.current = true;
        try {
          await getCurrentWindow().close();
        } catch {
          allowImmediateCloseRef.current = false;
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
          {saveLabel}
        </button>
        <button type="button" onClick={() => void saveProject(true)} disabled={busy}>
          Save As
        </button>
        <button type="button" onClick={() => void openProject()} disabled={busy}>
          Open
        </button>
        <button type="button" onClick={onOpenExport} disabled={busy}>
          Export
        </button>
        <button type="button" onClick={onOpenShortcuts} disabled={busy}>
          Shortcuts
        </button>
        <button type="button" onClick={() => void requestCloseApp()}>
          Quit
        </button>
      </div>

      <div className="project-menu-meta">
        <span>{getFileName(projectState.currentPath)}</span>
        <span>{formatAutosaveText(lastAutosaveAt)}</span>
      </div>

      {statusMessage ? <p className="project-status">{statusMessage}</p> : null}
      {error ? <p className="slot-error">{error}</p> : null}

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
