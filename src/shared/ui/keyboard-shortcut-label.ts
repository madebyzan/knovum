import { platform } from "@tauri-apps/plugin-os";

/**
 * Standard keyboard symbols aligned with Apple HIG, W3C, and Nuxt UI.
 */
const KBD_SYMBOLS = {
  command: "⌘",
  cmd: "⌘",
  option: "⌥",
  alt: "⌥",
  shift: "⇧",
  control: "⌃",
  ctrl: "⌃",
  win: "⊞",
  meta: "⌘",
  enter: "↵",
  return: "↵",
  backspace: "⌫",
  delete: "⌦",
  del: "⌦",
  escape: "Esc",
  esc: "Esc",
  tab: "⇥",
  capslock: "⇪",
  space: "Space",
  arrowup: "↑",
  up: "↑",
  arrowdown: "↓",
  down: "↓",
  arrowleft: "←",
  left: "←",
  arrowright: "→",
  right: "→",
  pageup: "⇞",
  pagedown: "⇟",
  home: "↖",
  end: "↘",
} as const;

const NON_MAC_OVERRIDE: Readonly<Record<string, string>> = {
  command: "Ctrl",
  cmd: "Ctrl",
  meta: "Ctrl",
  option: "Alt",
  alt: "Alt",
  control: "Ctrl",
  ctrl: "Ctrl",
  delete: "Del",
  del: "Del",
};

export interface KbdKeyGroup {
  connector?: string | undefined;
  keys: string[];
}

function isMac(): boolean {
  return platform() === "macos";
}

export function formatKey(key: string): string {
  const normalized = key.trim().toLowerCase();
  if (normalized === "mod" || normalized === "$mod") {
    return isMac() ? "⌘" : "Ctrl";
  }

  if (!isMac() && NON_MAC_OVERRIDE[normalized]) {
    return NON_MAC_OVERRIDE[normalized];
  }

  const symbol = KBD_SYMBOLS[normalized as keyof typeof KBD_SYMBOLS];
  if (symbol) {
    return symbol;
  }

  return key.length === 1 ? key.toUpperCase() : key;
}

export function parseShortcut(shortcut: string): KbdKeyGroup[] {
  if (!shortcut) return [];
  const segments = shortcut.split(/\s+(or|then)\s+/g);
  const groups: KbdKeyGroup[] = [];

  for (let i = 0; i < segments.length; i += 2) {
    const expr = segments[i]?.trim();
    if (!expr) continue;
    const connector = i > 0 ? segments[i - 1] : undefined;
    const keys = expr.replaceAll("$mod", "mod").split("+").map(formatKey);
    groups.push({ connector, keys });
  }

  return groups;
}

export function formatShortcutLabel(shortcut: string): string {
  const groups = parseShortcut(shortcut);
  if (!groups.length) return "";
  return groups
    .map((g) => `${g.connector ? ` ${g.connector} ` : ""}${g.keys.join(isMac() ? "" : "+")}`)
    .join("");
}
