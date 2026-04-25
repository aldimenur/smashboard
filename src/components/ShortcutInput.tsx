import { useEffect, useState } from "react";

import { normalizeShortcutString, shortcutFromKeyboardEvent } from "../utils/shortcut";

interface ShortcutInputProps {
  slotId: string;
  currentShortcut: string;
  onAssign: (slotId: string, shortcut: string) => Promise<void>;
}

export function ShortcutInput({ slotId, currentShortcut, onAssign }: ShortcutInputProps) {
  const [recording, setRecording] = useState(false);
  const [shortcut, setShortcut] = useState(normalizeShortcutString(currentShortcut));

  useEffect(() => {
    if (!recording) {
      setShortcut(normalizeShortcutString(currentShortcut));
    }
  }, [currentShortcut, recording]);

  return (
    <div className="shortcut-input-wrap">
      <input
        className="shortcut-input"
        value={recording ? "Press keys..." : shortcut || "None"}
        onFocus={() => setRecording(true)}
        onBlur={() => setRecording(false)}
        onKeyDown={(event) => {
          if (!recording) {
            return;
          }

          event.preventDefault();

          if (event.key === "Escape") {
            setRecording(false);
            return;
          }

          const assigned = shortcutFromKeyboardEvent(event.nativeEvent);
          if (!assigned) {
            return;
          }

          setShortcut(assigned);
          setRecording(false);
          void onAssign(slotId, assigned);
        }}
        readOnly
      />
      <button
        type="button"
        onClick={() => {
          setShortcut("");
          void onAssign(slotId, "");
        }}
      >
        Clear
      </button>
    </div>
  );
}
