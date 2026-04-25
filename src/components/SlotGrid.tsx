import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import { useSlots } from "../hooks/useSlots";
import type { Slot } from "../types";
import { normalizeShortcutString, shortcutFromKeyboardEvent } from "../utils/shortcut";
import { GlobalShortcutToggle } from "./GlobalShortcutToggle";
import { RecordingTransport } from "./RecordingTransport";
import { ShortcutInput } from "./ShortcutInput";
import { SlotCard } from "./SlotCard";
import { useMemo, useState } from "react";

const GRID_SIZE = 25;

export function SlotGrid() {
  const { slots, error, loadSlots, addSlot, triggerSlot, updateSlot, deleteSlot } = useSlots();
  const [editingSlotId, setEditingSlotId] = useState<string | null>(null);
  const [labelDraft, setLabelDraft] = useState("");
  const [pulseTicks, setPulseTicks] = useState<Record<string, number>>({});

  useEffect(() => {
    let unlistenProjectLoaded: UnlistenFn | undefined;
    let unlistenSlotTriggered: UnlistenFn | undefined;

    void loadSlots();

    void listen("project-loaded", () => {
      void loadSlots();
    }).then((fn) => {
      unlistenProjectLoaded = fn;
    });

    void listen<string>("slot-triggered", (event) => {
      const slotId = event.payload;
      setPulseTicks((prev) => ({
        ...prev,
        [slotId]: (prev[slotId] ?? 0) + 1,
      }));
    }).then((fn) => {
      unlistenSlotTriggered = fn;
    });

    return () => {
      unlistenProjectLoaded?.();
      unlistenSlotTriggered?.();
    };
  }, [loadSlots]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (target) {
        const tagName = target.tagName.toLowerCase();
        if (tagName === "input" || tagName === "textarea" || target.isContentEditable) {
          return;
        }
      }

      const shortcut = shortcutFromKeyboardEvent(event);
      if (!shortcut) {
        return;
      }

      const matchedSlot = slots.find(
        (slot) => normalizeShortcutString(slot.shortcut) === normalizeShortcutString(shortcut),
      );
      if (!matchedSlot) {
        return;
      }

      event.preventDefault();
      void triggerSlot(matchedSlot.id);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [slots, triggerSlot]);

  const editingSlot = useMemo(
    () => (editingSlotId ? slots.find((slot) => slot.id === editingSlotId) ?? null : null),
    [editingSlotId, slots],
  );

  useEffect(() => {
    if (editingSlot) {
      setLabelDraft(editingSlot.label);
    }
  }, [editingSlot]);

  const handleAddSlot = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Audio", extensions: ["wav", "mp3"] }],
      });

      if (typeof selected === "string") {
        await addSlot(selected);
      }
    } catch (err) {
      window.alert(`Failed to add slot: ${String(err)}`);
    }
  };

  const handleEdit = async (slot: Slot) => {
    setEditingSlotId(slot.id);
    setLabelDraft(slot.label);
  };

  const handleDelete = async (slotId: string) => {
    if (!window.confirm("Delete this slot?")) {
      return;
    }

    await deleteSlot(slotId);

    if (editingSlotId === slotId) {
      setEditingSlotId(null);
    }
  };

  const paddedSlots = Array.from({ length: GRID_SIZE }, (_, index) => slots[index]);

  return (
    <section className="slot-grid-wrapper">
      <div className="slot-grid-layout">
        <section className="slot-board-panel">
          <header className="slot-grid-toolbar">
            <div>
              <h1>SFX Board</h1>
              <p>{slots.length}/25 slots loaded</p>
            </div>

            <button type="button" onClick={() => void handleAddSlot()} disabled={slots.length >= GRID_SIZE}>
              Add Slot
            </button>
          </header>

          {editingSlot ? (
            <section className="slot-editor">
              <h2>Edit Slot</h2>
              <p className="slot-editor-hint">Each SFX slot can use its own custom shortcut.</p>
              <label>
                Label
                <input
                  value={labelDraft}
                  onChange={(event) => setLabelDraft(event.currentTarget.value)}
                  maxLength={64}
                />
              </label>

              <label>
                Shortcut
                <ShortcutInput
                  slotId={editingSlot.id}
                  currentShortcut={editingSlot.shortcut}
                  onAssign={async (slotId, shortcut) => {
                    await updateSlot(slotId, { shortcut: normalizeShortcutString(shortcut) });
                  }}
                />
              </label>

              <div className="slot-editor-actions">
                <button
                  type="button"
                  onClick={() => {
                    void updateSlot(editingSlot.id, { label: labelDraft.trim() || editingSlot.label });
                  }}
                >
                  Save Label
                </button>
                <button type="button" onClick={() => setEditingSlotId(null)}>
                  Close
                </button>
              </div>
            </section>
          ) : null}

          {error ? <p className="slot-error">{error}</p> : null}

          <div className="slot-grid">
            {paddedSlots.map((slot, index) => (
              <SlotCard
                key={slot?.id ?? `empty-${index}`}
                index={index}
                slot={slot}
                pulseTick={slot ? pulseTicks[slot.id] ?? 0 : 0}
                onTrigger={triggerSlot}
                onEdit={handleEdit}
                onDelete={handleDelete}
                onGainChange={async (slotId, gain) => {
                  await updateSlot(slotId, { gain });
                }}
              />
            ))}
          </div>
        </section>

        <aside className="recording-panel">
          <h2>Recording Transport</h2>
          <RecordingTransport />
          <GlobalShortcutToggle />
        </aside>
      </div>
    </section>
  );
}
