import { describe, expect, it } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { collectShortcuts } from "../shortcut-modal-items";

function dummyTranslate(key: string): string {
  return `translated_${key}`;
}

describe("shortcut modal items", () => {
  it("complements active commands with unmounted catalog commands", () => {
    const activeCommands: AppCommand[] = [
      {
        id: "shell.command-palette",
        title: "Palette",
        shortcut: "$mod+K",
        category: "general",
        run: () => {},
      },
    ];

    const shortcuts = collectShortcuts(activeCommands, dummyTranslate, (id, def) =>
      id === "shell.command-palette" ? "$mod+P" : def,
    );

    // Active command should be present with effective shortcut
    const palette = shortcuts.find((item) => item.id === "shell.command-palette");
    expect(palette).toBeDefined();
    expect(palette?.shortcut).toBe("$mod+P");

    // Catalog commands not in activeCommands should also be present
    const toolSelect = shortcuts.find((item) => item.id === "spaces.tool-select");
    expect(toolSelect).toBeDefined();
    expect(toolSelect?.shortcut).toBe("V");
    expect(toolSelect?.title).toBe("translated_spaces.canvas.tools.select");
  });

  it("handles grouped commands like nudge", () => {
    const activeCommands: AppCommand[] = [
      {
        id: "spaces.nudge-left",
        title: "Left",
        shortcut: "ArrowLeft",
        shortcutListing: {
          groupId: "canvas.nudge",
          title: "Nudge Selection",
          shortcut: "Arrow keys",
        },
        category: "canvas",
        run: () => {},
      },
      {
        id: "spaces.nudge-right",
        title: "Right",
        shortcut: "ArrowRight",
        shortcutListing: {
          groupId: "canvas.nudge",
          title: "Nudge Selection",
          shortcut: "Arrow keys",
        },
        category: "canvas",
        run: () => {},
      },
    ];

    const shortcuts = collectShortcuts(activeCommands, dummyTranslate, (_id, def) => def);
    const nudge = shortcuts.filter((item) => item.id === "canvas.nudge");
    expect(nudge.length).toBe(1);
    expect(nudge[0]?.title).toBe("Nudge Selection");
  });
});
