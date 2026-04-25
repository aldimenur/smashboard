import { useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import { useSlots } from "../hooks/useSlots";
import type { Slot } from "../types";
import { SlotCard } from "./SlotCard";

const GRID_SIZE = 64;

export function SlotGrid() {
  const { slots, error, loadSlots, addSlot, triggerSlot, updateSlot, deleteSlot } = useSlots();

  useEffect(() => {
    void loadSlots();
  }, [loadSlots]);

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
    const nextLabel = window.prompt("Slot label", slot.label);
    if (nextLabel === null) {
      return;
    }

    const nextShortcut = window.prompt("Shortcut", slot.shortcut);
    if (nextShortcut === null) {
      return;
    }

    await updateSlot(slot.id, { label: nextLabel.trim(), shortcut: nextShortcut.trim() });
  };

  const handleDelete = async (slotId: string) => {
    if (!window.confirm("Delete this slot?")) {
      return;
    }

    await deleteSlot(slotId);
  };

  const paddedSlots = Array.from({ length: GRID_SIZE }, (_, index) => slots[index]);

  return (
    <section className="slot-grid-wrapper">
      <header className="slot-grid-toolbar">
        <div>
          <h1>SFX Board</h1>
          <p>{slots.length}/64 slots loaded</p>
        </div>

        <button type="button" onClick={() => void handleAddSlot()} disabled={slots.length >= GRID_SIZE}>
          Add Slot
        </button>
      </header>

      {error ? <p className="slot-error">{error}</p> : null}

      <div className="slot-grid">
        {paddedSlots.map((slot, index) => (
          <SlotCard
            key={slot?.id ?? `empty-${index}`}
            index={index}
            slot={slot}
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
  );
}
