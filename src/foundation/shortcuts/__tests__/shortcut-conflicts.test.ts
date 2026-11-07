import { describe, expect, it } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { findShortcutConflict } from "../shortcut-conflicts";

describe("shortcut-conflicts", () => {
  const commands: AppCommand[] = [
    {
      id: "shell.commandPalette",
      title: "Command Palette",
      shortcut: "Mod+K",
      run: () => {},
    },
    {
      id: "shell.shortcuts",
      title: "Keyboard Shortcuts",
      shortcut: "Mod+/",
      run: () => {},
    },
  ];

  it("returns undefined when no conflict exists", () => {
    const conflict = findShortcutConflict("shell.commandPalette", "Mod+P", commands, {});
    expect(conflict).toBeUndefined();
  });

  it("detects conflict with default command shortcut", () => {
    const conflict = findShortcutConflict("other.command", "Mod+K", commands, {});
    expect(conflict).toBeDefined();
    expect(conflict?.id).toBe("shell.commandPalette");
  });

  it("ignores conflict with self", () => {
    const conflict = findShortcutConflict("shell.commandPalette", "Mod+K", commands, {});
    expect(conflict).toBeUndefined();
  });

  it("detects conflict with customized shortcut", () => {
    const customShortcuts = {
      "shell.shortcuts": "Mod+Shift+P",
    };
    const conflict = findShortcutConflict("other.command", "Mod+Shift+P", commands, customShortcuts);
    expect(conflict).toBeDefined();
    expect(conflict?.id).toBe("shell.shortcuts");
  });

  it("matches normalized chords with different component orders", () => {
    const conflict = findShortcutConflict("other.command", "K+Mod", commands, {});
    expect(conflict).toBeDefined();
    expect(conflict?.id).toBe("shell.commandPalette");
  });
});
