import { platform } from "@tauri-apps/plugin-os";

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

const SPECIAL_KEY_MAP: Record<string, string> = {
  " ": "Space",
  Spacebar: "Space",
  Escape: "Escape",
  Esc: "Escape",
  Enter: "Enter",
  Return: "Enter",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  Del: "Delete",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
};

export function isModifierOnlyKey(key: string): boolean {
  return MODIFIER_KEYS.has(key);
}

function detectIsMac(): boolean {
  try {
    return platform() === "macos";
  } catch {
    return typeof navigator !== "undefined" && /Mac|iPod|iPhone|iPad/.test(navigator.platform);
  }
}

function collectModifiers(event: KeyboardEvent, isMac: boolean): string[] {
  const parts: string[] = [];
  const modPressed = isMac ? event.metaKey : event.ctrlKey;
  if (modPressed) parts.push("$mod");
  if (isMac && event.ctrlKey) parts.push("Control");
  if (!isMac && event.metaKey) parts.push("Meta");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  return parts;
}

function normalizeKey(rawKey: string): string {
  return SPECIAL_KEY_MAP[rawKey] ?? (rawKey.length === 1 ? rawKey.toUpperCase() : rawKey);
}

export function parseKeyboardEventToChord(event: KeyboardEvent): string | null {
  if (isModifierOnlyKey(event.key)) return null;
  const isMac = detectIsMac();
  const parts = collectModifiers(event, isMac);
  parts.push(normalizeKey(event.key));
  return parts.join("+");
}

export function toGlobalAccelerator(chord: string): string {
  return chord
    .replace(/\$mod/gi, "CommandOrControl")
    .replace(/\bmod\b/gi, "CommandOrControl")
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean)
    .join("+");
}

export function normalizeChord(chord: string): string {
  const parts = chord
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean);
  if (!parts.length) return "";

  const hasMod = parts.some((p) => p === "$mod" || p.toLowerCase() === "mod");
  const hasCtrl = parts.some((p) => p.toLowerCase() === "control" || p.toLowerCase() === "ctrl");
  const hasAlt = parts.some((p) => p.toLowerCase() === "alt" || p.toLowerCase() === "option");
  const hasShift = parts.some((p) => p.toLowerCase() === "shift");
  const hasMeta = parts.some((p) => p.toLowerCase() === "meta" || p.toLowerCase() === "cmd");

  const nonModifiers = parts.filter(
    (p) =>
      !["$mod", "mod", "control", "ctrl", "alt", "option", "shift", "meta", "cmd"].includes(p.toLowerCase()),
  );

  const ordered: string[] = [];
  if (hasMod) ordered.push("$mod");
  else if (hasCtrl) ordered.push("Control");
  if (hasAlt) ordered.push("Alt");
  if (hasShift) ordered.push("Shift");
  if (hasMeta && !hasMod) ordered.push("Meta");

  ordered.push(...nonModifiers.map((k) => (k.length === 1 ? k.toUpperCase() : k)));
  return ordered.join("+");
}
