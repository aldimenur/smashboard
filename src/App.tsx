import "./App.css";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { confirm, message } from "@tauri-apps/plugin-dialog";

import { ExportDialog } from "./components/ExportDialog";
import { KeyboardShortcuts } from "./components/KeyboardShortcuts";
import { ProjectMenu } from "./components/ProjectMenu";
import { SlotGrid } from "./components/SlotGrid";
import { TimelinePanel } from "./components/Timeline/TimelinePanel";
import { ToastContainer, useToast } from "./components/Toast";
import type { AutosaveRecoveryInfo } from "./types";

function App() {
  const [exportOpen, setExportOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [projectName, setProjectName] = useState("Untitled");
  const { toasts, showToast } = useToast();

  useEffect(() => {
    const checkRecovery = async () => {
      try {
        const recovery = await invoke<AutosaveRecoveryInfo>("check_autosave_recovery");
        if (!recovery.hasRecoverable) {
          return;
        }

        const modifiedText = recovery.modifiedAt
          ? new Date(recovery.modifiedAt).toLocaleString()
          : "unknown time";
        const shouldRecover = await confirm(
          `Recover unsaved work from autosave?\nLast autosave: ${modifiedText}`,
          {
            title: "Autosave Recovery",
            kind: "warning",
            okLabel: "Recover",
            cancelLabel: "Discard",
          },
        );

        if (!shouldRecover) {
          return;
        }

        await invoke("load_project", { filePath: recovery.autosavePath });
        showToast("Recovered project from autosave", "success");

        const missing = await invoke<string[]>("validate_audio_paths");
        if (missing.length > 0) {
          await message(
            `Recovered project has missing files:\n${missing.slice(0, 8).join("\n")}${
              missing.length > 8 ? `\n...and ${missing.length - 8} more` : ""
            }`,
            {
              title: "Recovered With Missing Files",
              kind: "warning",
            },
          );
        }
      } catch {
        // Recovery is best effort; keep startup resilient.
      }
    };

    void checkRecovery();
  }, [showToast]);

  return (
    <main className="app-shell">
      <ProjectMenu
        onOpenExport={() => setExportOpen(true)}
        onOpenShortcuts={() => setShortcutsOpen(true)}
        onProjectNameChange={setProjectName}
        onToast={showToast}
      />
      <SlotGrid />
      <TimelinePanel onToast={showToast} />
      <ExportDialog
        openState={exportOpen}
        projectName={projectName}
        onClose={() => setExportOpen(false)}
        onToast={showToast}
      />
      <KeyboardShortcuts openState={shortcutsOpen} onClose={() => setShortcutsOpen(false)} />
      <ToastContainer toasts={toasts} />
    </main>
  );
}

export default App;
