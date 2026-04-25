import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFloppyDisk, faGear, faXmark } from "@fortawesome/free-solid-svg-icons";

import { useSlots } from "../hooks/useSlots";
import type { Slot } from "../types";
import { normalizeShortcutString, shortcutFromKeyboardEvent } from "../utils/shortcut";
import { GlobalShortcutToggle } from "./GlobalShortcutToggle";
import { RecordingTransport } from "./RecordingTransport";
import { ShortcutInput } from "./ShortcutInput";
import { SlotCard } from "./SlotCard";
import { useMemo, useState } from "react";

export function SlotGrid() {
  const { slots, error, loadSlots, addSlotAtPosition, triggerSlot, updateSlot, deleteSlot } = useSlots();
  const [editingSlotId, setEditingSlotId] = useState<string | null>(null);
  const [labelDraft, setLabelDraft] = useState("");
  const [pulseTicks, setPulseTicks] = useState<Record<string, number>>({});
  const [boardRows, setBoardRows] = useState(5);
  const [boardColumns, setBoardColumns] = useState(5);
  const [layoutError, setLayoutError] = useState<string | null>(null);

  useEffect(() => {
    let unlistenProjectLoaded: UnlistenFn | undefined;
    let unlistenSlotTriggered: UnlistenFn | undefined;

    const loadBoardLayout = async () => {
      try {
        const state = await invoke<{ boardRows: number; boardColumns: number }>("get_project_state");
        setBoardRows(state.boardRows);
        setBoardColumns(state.boardColumns);
      } catch {
        setBoardRows(5);
        setBoardColumns(5);
      }
    };

    void loadSlots();
    void loadBoardLayout();

    void listen("project-loaded", () => {
      void loadSlots();
      void loadBoardLayout();
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

  const handleImportAtPosition = async (position: number) => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Audio", extensions: ["wav", "mp3"] }],
      });

      if (typeof selected === "string") {
        await addSlotAtPosition(selected, position);
        setLayoutError(null);
      }
    } catch (err) {
      setLayoutError(String(err));
    }
  };

  const handleBoardRowsChange = async (rows: number) => {
    try {
      await invoke("update_board_layout", { rows, columns: boardColumns });
      setBoardRows(rows);
      setLayoutError(null);
    } catch (err) {
      setLayoutError(String(err));
    }
  };

  const handleBoardColumnsChange = async (columns: number) => {
    try {
      await invoke("update_board_layout", { rows: boardRows, columns });
      setBoardColumns(columns);
      setLayoutError(null);
    } catch (err) {
      setLayoutError(String(err));
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

  const gridCapacity = boardRows * boardColumns;
  const slotsByPosition = useMemo(() => {
    const map = new Map<number, Slot>();
    for (const slot of slots) {
      map.set(slot.position, slot);
    }
    return map;
  }, [slots]);

  const paddedSlots = Array.from({ length: gridCapacity }, (_, index) => slotsByPosition.get(index));

  return (
    <section className="slot-grid-wrapper">
      <div className="slot-grid-layout">
        <section className="slot-board-panel">
          <header className="slot-grid-toolbar">
            <div className="board-settings">
              <span className="board-settings-title">
                <FontAwesomeIcon icon={faGear} />
                Board
              </span>
              <label>
                Rows
                <select value={boardRows} onChange={(event) => void handleBoardRowsChange(Number(event.currentTarget.value))}>
                  {Array.from({ length: 5 }, (_, index) => index + 1).map((value) => (
                    <option key={`row-${value}`} value={value}>
                      {value}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Cols
                <select
                  value={boardColumns}
                  onChange={(event) => void handleBoardColumnsChange(Number(event.currentTarget.value))}
                >
                  {Array.from({ length: 5 }, (_, index) => index + 1).map((value) => (
                    <option key={`col-${value}`} value={value}>
                      {value}
                    </option>
                  ))}
                </select>
              </label>
            </div>
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
                  <FontAwesomeIcon icon={faFloppyDisk} />
                  Save Label
                </button>
                <button type="button" onClick={() => setEditingSlotId(null)}>
                  <FontAwesomeIcon icon={faXmark} />
                  Close
                </button>
              </div>
            </section>
          ) : null}

          {error ? <p className="slot-error">{error}</p> : null}
          {layoutError ? <p className="slot-error">{layoutError}</p> : null}

          <div className="slot-grid" style={{ gridTemplateColumns: `repeat(${boardColumns}, minmax(0, 1fr))` }}>
            {paddedSlots.map((slot, index) => (
              <SlotCard
                key={slot?.id ?? `empty-${index}`}
                index={index}
                slot={slot}
                pulseTick={slot ? pulseTicks[slot.id] ?? 0 : 0}
                onImport={handleImportAtPosition}
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
