import { describe, expect, it, vi } from "vitest";

import { formatKey, formatShortcutLabel, parseShortcut } from "../keyboard-shortcut-label";

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: () => "macos",
}));

describe("keyboard shortcut label", () => {
  it("formats standard modifiers on macOS", () => {
    expect(formatKey("mod")).toBe("⌘");
    expect(formatKey("shift")).toBe("⇧");
    expect(formatKey("alt")).toBe("⌥");
    expect(formatKey("control")).toBe("⌃");
    expect(formatKey("escape")).toBe("Esc");
    expect(formatKey("enter")).toBe("↵");
    expect(formatShortcutLabel("$mod+k")).toBe("⌘K");
  });

  it("parses multi-step shortcuts with connectors", () => {
    const groups = parseShortcut("$mod+k or $mod+p");
    expect(groups).toHaveLength(2);
    expect(groups[0]?.keys).toEqual(["⌘", "K"]);
    expect(groups[1]?.connector).toBe("or");
    expect(groups[1]?.keys).toEqual(["⌘", "P"]);
  });
});
