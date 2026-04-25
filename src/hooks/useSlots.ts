import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import type { Slot } from "../types";

interface UpdateSlotPayload {
  label?: string;
  shortcut?: string;
  gain?: number;
}

export function useSlots() {
  const [slots, setSlots] = useState<Slot[]>([]);
  const [error, setError] = useState<string | null>(null);

  const loadSlots = useCallback(async () => {
    try {
      const result = await invoke<Slot[]>("get_all_slots");
      setSlots(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const addSlot = useCallback(async (filePath: string) => {
    try {
      const slot = await invoke<Slot>("add_slot", { filePath });
      setSlots((prev) => [...prev, slot]);
      setError(null);
      return slot;
    } catch (err) {
      setError(String(err));
      throw err;
    }
  }, []);

  const triggerSlot = useCallback(async (slotId: string) => {
    try {
      await invoke("trigger_slot", { slotId });
      setError(null);
    } catch (err) {
      setError(String(err));
      throw err;
    }
  }, []);

  const updateSlot = useCallback(async (slotId: string, payload: UpdateSlotPayload) => {
    try {
      const updated = await invoke<Slot>("update_slot", {
        slotId,
        label: payload.label,
        shortcut: payload.shortcut,
        gain: payload.gain,
      });

      setSlots((prev) => prev.map((slot) => (slot.id === slotId ? updated : slot)));
      setError(null);
      return updated;
    } catch (err) {
      setError(String(err));
      throw err;
    }
  }, []);

  const deleteSlot = useCallback(async (slotId: string) => {
    try {
      await invoke("delete_slot", { slotId });
      setSlots((prev) => prev.filter((slot) => slot.id !== slotId));
      setError(null);
    } catch (err) {
      setError(String(err));
      throw err;
    }
  }, []);

  return {
    slots,
    error,
    loadSlots,
    addSlot,
    triggerSlot,
    updateSlot,
    deleteSlot,
  };
}
