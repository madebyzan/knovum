import { describe, expect, it, vi } from "vitest";

import {
  fromGlobalAccelerator,
  isModifierOnlyKey,
  normalizeChord,
  parseKeyboardEventToChord,
  toGlobalAccelerator,
} from "../shortcut-chord";

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: () => "macos",
}));

describe("shortcut-chord", () => {
  it("detects modifier-only keys", () => {
    expect(isModifierOnlyKey("Control")).toBe(true);
    expect(isModifierOnlyKey("Shift")).toBe(true);
    expect(isModifierOnlyKey("Alt")).toBe(true);
    expect(isModifierOnlyKey("Meta")).toBe(true);
    expect(isModifierOnlyKey("k")).toBe(false);
    expect(isModifierOnlyKey("Enter")).toBe(false);
  });

  it("normalizes chords into consistent modifier order", () => {
    expect(normalizeChord("k+Mod")).toBe("$mod+K");
    expect(normalizeChord("Shift+Mod+P")).toBe("$mod+Shift+P");
    expect(normalizeChord("Alt+Shift+Mod+X")).toBe("$mod+Alt+Shift+X");
    expect(normalizeChord("CommandOrControl+Shift+Space")).toBe("$mod+Shift+Space");
  });

  it("translates chords to and from global accelerators", () => {
    expect(toGlobalAccelerator("$mod+Shift+Space")).toBe("CommandOrControl+Shift+Space");
    expect(toGlobalAccelerator("Mod+Shift+Space")).toBe("CommandOrControl+Shift+Space");
    expect(toGlobalAccelerator("$mod+Alt+K")).toBe("CommandOrControl+Alt+K");
    expect(fromGlobalAccelerator("CommandOrControl+Shift+Space")).toBe("$mod+Shift+Space");
  });

  it("parses keyboard events into normalized chords", () => {
    const event1 = new KeyboardEvent("keydown", {
      key: "k",
      metaKey: true,
    });
    expect(parseKeyboardEventToChord(event1)).toBe("$mod+K");

    const event2 = new KeyboardEvent("keydown", {
      key: "p",
      metaKey: true,
      shiftKey: true,
    });
    expect(parseKeyboardEventToChord(event2)).toBe("$mod+Shift+P");

    const event3 = new KeyboardEvent("keydown", {
      key: "F1",
    });
    expect(parseKeyboardEventToChord(event3)).toBe("F1");

    // Modifier only returns null
    const modifierEvent = new KeyboardEvent("keydown", {
      key: "Shift",
      shiftKey: true,
    });
    expect(parseKeyboardEventToChord(modifierEvent)).toBeNull();
  });
});
