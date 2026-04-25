const MODIFIER_KEYS = new Set(["Control", "Alt", "Shift", "Meta"]);

function normalizeMainKey(key: string): string {
  if (key === " ") {
    return "Space";
  }

  if (key.length === 1) {
    return key.toUpperCase();
  }

  if (key === "Esc") {
    return "Escape";
  }

  if (key === "OS") {
    return "Super";
  }

  return key;
}

function normalizeModifier(token: string): string | null {
  const upper = token.trim().toUpperCase();

  if (upper === "CTRL" || upper === "CONTROL") {
    return "Ctrl";
  }

  if (upper === "ALT" || upper === "OPTION") {
    return "Alt";
  }

  if (upper === "SHIFT") {
    return "Shift";
  }

  if (upper === "META" || upper === "SUPER" || upper === "CMD" || upper === "COMMAND") {
    return "Super";
  }

  return null;
}

export function normalizeShortcutString(input: string): string {
  const tokens = input
    .split("+")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);

  if (tokens.length === 0) {
    return "";
  }

  const modifiers = new Set<string>();
  let key: string | null = null;

  for (const token of tokens) {
    const modifier = normalizeModifier(token);
    if (modifier) {
      modifiers.add(modifier);
      continue;
    }

    key = normalizeMainKey(token);
  }

  if (!key) {
    return "";
  }

  const orderedModifiers = ["Ctrl", "Alt", "Shift", "Super"].filter((value) =>
    modifiers.has(value),
  );

  return [...orderedModifiers, key].join("+");
}

export function shortcutFromKeyboardEvent(event: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(event.key)) {
    return null;
  }

  const key = normalizeMainKey(event.key);
  const parts: string[] = [];

  if (event.ctrlKey) {
    parts.push("Ctrl");
  }

  if (event.altKey) {
    parts.push("Alt");
  }

  if (event.shiftKey) {
    parts.push("Shift");
  }

  if (event.metaKey) {
    parts.push("Super");
  }

  parts.push(key);

  return normalizeShortcutString(parts.join("+"));
}
