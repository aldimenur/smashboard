import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFloppyDisk, faGear, faImage, faUpload, faXmark } from "@fortawesome/free-solid-svg-icons";

import { useSlots } from "../hooks/useSlots";
import type { Slot } from "../types";
import { normalizeShortcutString, shortcutFromKeyboardEvent } from "../utils/shortcut";
import { SLOT_ICON_OPTIONS } from "../utils/slotIcons";
import { RecordingTransport } from "./RecordingTransport";
import { ShortcutInput } from "./ShortcutInput";
import { SlotCard } from "./SlotCard";
import { useMemo, useState } from "react";

interface SlotImageLoadResult {
  mimeType: string;
  bytes: number[];
}

function encodeImageDataUrl(bytes: number[], mimeType: string): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let index = 0; index < bytes.length; index += chunkSize) {
    const chunk = bytes.slice(index, index + chunkSize);
    binary += String.fromCharCode(...chunk);
  }
  return `data:${mimeType};base64,${btoa(binary)}`;
}

export function SlotGrid() {
  const { slots, error, loadSlots, addSlotAtPosition, triggerSlot, updateSlot, deleteSlot } = useSlots();
  const [editingSlotId, setEditingSlotId] = useState<string | null>(null);
  const [labelDraft, setLabelDraft] = useState("");
  const [imageDataDraft, setImageDataDraft] = useState<string | null>(null);
  const [iconNameDraft, setIconNameDraft] = useState<string | null>(null);
  const [pulseTicks, setPulseTicks] = useState<Record<string, number>>({});
  const [boardRows, setBoardRows] = useState(5);
  const [boardColumns, setBoardColumns] = useState(5);
  const [boardLabel, setBoardLabel] = useState("SFX Board");
  const [boardLabelDraft, setBoardLabelDraft] = useState("SFX Board");
  const [layoutError, setLayoutError] = useState<string | null>(null);

  useEffect(() => {
    let unlistenProjectLoaded: UnlistenFn | undefined;
    let unlistenSlotTriggered: UnlistenFn | undefined;

    const loadBoardLayout = async () => {
      try {
        const state = await invoke<{ boardRows: number; boardColumns: number; boardLabel: string }>("get_project_state");
        setBoardRows(state.boardRows);
        setBoardColumns(state.boardColumns);
        setBoardLabel(state.boardLabel);
        setBoardLabelDraft(state.boardLabel);
      } catch {
        setBoardRows(5);
        setBoardColumns(5);
        setBoardLabel("SFX Board");
        setBoardLabelDraft("SFX Board");
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
      setImageDataDraft(editingSlot.imageDataUrl ?? null);
      setIconNameDraft(editingSlot.iconName ?? null);
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

  const saveBoardLabel = async () => {
    try {
      await invoke("update_board_label", { label: boardLabelDraft });
      setBoardLabel(boardLabelDraft.trim());
      setLayoutError(null);
    } catch (err) {
      setLayoutError(String(err));
    }
  };

  const handleEdit = async (slot: Slot) => {
    setEditingSlotId(slot.id);
    setLabelDraft(slot.label);
    setImageDataDraft(slot.imageDataUrl ?? null);
    setIconNameDraft(slot.iconName ?? null);
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
  const selectedIcon = useMemo(
    () => SLOT_ICON_OPTIONS.find((option) => option.name === iconNameDraft)?.icon ?? null,
    [iconNameDraft],
  );

  const pickSlotImage = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif", "svg"] }],
    });

    if (typeof selected !== "string") {
      return;
    }

    const payload = await invoke<SlotImageLoadResult>("load_slot_image_data", { filePath: selected });
    const imageDataUrl = encodeImageDataUrl(payload.bytes, payload.mimeType);
    setImageDataDraft(imageDataUrl);
    setIconNameDraft(null);
    setLayoutError(null);
  };

  const uploadCustomImage = async () => {
    try {
      await pickSlotImage();
    } catch (err) {
      setLayoutError(String(err));
    }
  };

  return (
    <section className="slot-grid-wrapper">
      <div className="slot-grid-layout">
        <section className="slot-board-panel">
          <header className="slot-grid-toolbar">
            <div className="board-settings">
              <span className="board-settings-title">
                <FontAwesomeIcon icon={faGear} />
                {boardLabel}
              </span>
              <label>
                Label
                <input
                  className="board-label-input"
                  value={boardLabelDraft}
                  onChange={(event) => setBoardLabelDraft(event.currentTarget.value)}
                  onBlur={() => void saveBoardLabel()}
                  maxLength={48}
                />
              </label>
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
        </aside>
      </div>

      {editingSlot ? (
        <section
          className="dialog-backdrop"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) {
              setEditingSlotId(null);
            }
          }}
        >
          <div className="dialog-card slot-editor-modal" onMouseDown={(event) => event.stopPropagation()}>
            <h2>Edit Slot</h2>
            <p className="slot-editor-hint">Update label, shortcut, and optional slot image.</p>
            <label>
              Label
              <input value={labelDraft} onChange={(event) => setLabelDraft(event.currentTarget.value)} maxLength={64} />
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

            <div className="slot-image-editor">
              <span className="slot-image-editor-label">Artwork</span>
              <div className="slot-image-editor-preview">
                {imageDataDraft ? (
                  <img src={imageDataDraft} alt="" />
                ) : selectedIcon ? (
                  <FontAwesomeIcon icon={selectedIcon} />
                ) : (
                  <FontAwesomeIcon icon={faImage} />
                )}
              </div>
              <label>
                Icon Library (Font Awesome)
                <select
                  value={iconNameDraft ?? ""}
                  onChange={(event) => {
                    const value = event.currentTarget.value.trim();
                    setIconNameDraft(value || null);
                    if (value) {
                      setImageDataDraft(null);
                    }
                  }}
                >
                  <option value="">None</option>
                  {SLOT_ICON_OPTIONS.map((option) => (
                    <option key={option.name} value={option.name}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
              <div className="slot-image-editor-actions">
                <button type="button" onClick={() => void uploadCustomImage()}>
                  <FontAwesomeIcon icon={faUpload} />
                  Upload Image
                </button>
                <button
                  type="button"
                  className="button-danger-soft"
                  onClick={() => {
                    setImageDataDraft(null);
                    setIconNameDraft(null);
                  }}
                >
                  Remove
                </button>
              </div>
            </div>

            <div className="slot-editor-actions">
              <button
                type="button"
                onClick={async () => {
                  try {
                    const payload: { label: string; imageDataUrl?: string; iconName?: string } = {
                      label: labelDraft.trim() || editingSlot.label,
                    };
                    if (imageDataDraft !== (editingSlot.imageDataUrl ?? null)) {
                      payload.imageDataUrl = imageDataDraft ?? "";
                    }
                    if (iconNameDraft !== (editingSlot.iconName ?? null)) {
                      payload.iconName = iconNameDraft ?? "";
                    }

                    await updateSlot(editingSlot.id, payload);
                    setEditingSlotId(null);
                  } catch {
                    // Error already surfaced by hook state.
                  }
                }}
              >
                <FontAwesomeIcon icon={faFloppyDisk} />
                Save
              </button>
              <button type="button" onClick={() => setEditingSlotId(null)}>
                <FontAwesomeIcon icon={faXmark} />
                Close
              </button>
            </div>
          </div>
        </section>
      ) : null}
    </section>
  );
}
