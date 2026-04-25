import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function GlobalShortcutToggle() {
  const [enabled, setEnabled] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void invoke<boolean>("get_global_shortcuts_enabled")
      .then((value) => {
        setEnabled(value);
        setError(null);
      })
      .catch((err) => {
        setError(String(err));
      });
  }, []);

  const toggle = async () => {
    const nextEnabled = !enabled;

    try {
      await invoke("set_global_shortcuts_enabled", { enabled: nextEnabled });
      setEnabled(nextEnabled);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <section className="global-shortcut-status">
      <button type="button" onClick={() => void toggle()}>
        {enabled ? "● Active" : "○ Inactive"}
      </button>
      <span>Global Shortcuts</span>
      {error ? <span className="shortcut-error">{error}</span> : null}
    </section>
  );
}
