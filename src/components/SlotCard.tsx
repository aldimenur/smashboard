import { useEffect, useMemo, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faMusic, faPenToSquare, faTrash } from "@fortawesome/free-solid-svg-icons";

import type { Slot } from "../types";

interface SlotCardProps {
  index: number;
  slot?: Slot;
  pulseTick?: number;
  onImport: (position: number) => Promise<void>;
  onTrigger: (slotId: string) => Promise<void>;
  onEdit: (slot: Slot) => Promise<void>;
  onDelete: (slotId: string) => Promise<void>;
  onGainChange: (slotId: string, gain: number) => Promise<void>;
}

export function SlotCard({
  index,
  slot,
  pulseTick = 0,
  onImport,
  onTrigger,
  onEdit,
  onDelete,
  onGainChange,
}: SlotCardProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    if (!menuOpen) {
      return undefined;
    }

    const closeMenu = () => setMenuOpen(false);
    window.addEventListener("click", closeMenu);
    return () => window.removeEventListener("click", closeMenu);
  }, [menuOpen]);

  useEffect(() => {
    if (!slot || pulseTick <= 0) {
      return;
    }

    setIsPlaying(true);
    const timeout = window.setTimeout(() => setIsPlaying(false), 240);
    return () => window.clearTimeout(timeout);
  }, [pulseTick, slot]);

  const durationText = useMemo(() => {
    if (!slot) {
      return "";
    }

    return `${(slot.durationMs / 1000).toFixed(2)}s`;
  }, [slot]);

  const handleTrigger = async () => {
    if (!slot) {
      return;
    }

    try {
      await onTrigger(slot.id);
    } catch {
      setIsPlaying(false);
    }
  };

  const style = slot
    ? ({ "--slot-color": slot.color } as React.CSSProperties)
    : ({ "--slot-color": "#3A3A3A" } as React.CSSProperties);

  if (!slot) {
    return (
      <article
        className="slot-card slot-empty slot-empty-import"
        style={style}
        role="button"
        tabIndex={0}
        onClick={() => void onImport(index)}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            void onImport(index);
          }
        }}
      >
        <span className="slot-index">{index + 1}</span>
        <span className="slot-empty-label">
          <FontAwesomeIcon icon={faMusic} />
          Import
        </span>
      </article>
    );
  }

  return (
    <article
      className={`slot-card slot-loaded ${isPlaying ? "slot-playing" : ""}`}
      style={style}
      onClick={handleTrigger}
      onContextMenu={(event) => {
        event.preventDefault();
        setMenuOpen(true);
      }}
      role="button"
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          void handleTrigger();
        }
      }}
      aria-label={`Trigger slot ${slot.label}`}
    >
      <header className="slot-head">
        <span className="slot-index">{index + 1}</span>
        <span className="slot-shortcut">{slot.shortcut || "--"}</span>
      </header>

      <strong className="slot-label" title={slot.label}>
        {slot.label}
      </strong>

      <div className="slot-meta">
        <span>{durationText}</span>
        <span>{Math.round(slot.gain * 100)}%</span>
      </div>

      <label className="slot-gain">
        <span>Gain</span>
        <input
          type="range"
          min={0}
          max={2}
          step={0.05}
          value={slot.gain}
          onClick={(event) => event.stopPropagation()}
          onMouseDown={(event) => event.stopPropagation()}
          onChange={(event) => {
            const gain = Number.parseFloat(event.currentTarget.value);
            void onGainChange(slot.id, gain);
          }}
        />
      </label>

      {menuOpen ? (
        <div
          className="slot-context-menu"
          onClick={(event) => event.stopPropagation()}
          onMouseDown={(event) => event.stopPropagation()}
        >
          <button
            type="button"
            onClick={() => {
              setMenuOpen(false);
              void onEdit(slot);
            }}
          >
            <FontAwesomeIcon icon={faPenToSquare} />
            Edit
          </button>
          <button
            type="button"
            onClick={() => {
              setMenuOpen(false);
              void onDelete(slot.id);
            }}
          >
            <FontAwesomeIcon icon={faTrash} />
            Delete
          </button>
        </div>
      ) : null}
    </article>
  );
}
