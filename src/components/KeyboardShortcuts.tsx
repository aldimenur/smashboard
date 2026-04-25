interface ShortcutItem {
  keys: string;
  action: string;
}

interface ShortcutCategory {
  category: string;
  items: ShortcutItem[];
}

interface KeyboardShortcutsProps {
  openState: boolean;
  onClose: () => void;
}

const SHORTCUTS: ShortcutCategory[] = [
  {
    category: "SFX Board",
    items: [
      { keys: "Assigned Slot Shortcut", action: "Trigger slot soundboard action" },
      { keys: "Click Slot", action: "Trigger slot with mouse" },
    ],
  },
];

export function KeyboardShortcuts({ openState, onClose }: KeyboardShortcutsProps) {
  if (!openState) {
    return null;
  }

  return (
    <section className="dialog-backdrop">
      <div className="dialog-card shortcuts-dialog">
        <h2>Keyboard Shortcuts</h2>
        <p>Playback, timeline, recording, and project actions use UI buttons only.</p>

        <div className="shortcuts-grid">
          {SHORTCUTS.map((category) => (
            <div key={category.category} className="shortcut-category">
              <h3>{category.category}</h3>
              <table>
                <tbody>
                  {category.items.map((item) => (
                    <tr key={item.keys}>
                      <td className="shortcut-keys">
                        <kbd>{item.keys}</kbd>
                      </td>
                      <td className="shortcut-action">{item.action}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ))}
        </div>

        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </section>
  );
}
