import { describe, expect, it, vi } from "vitest";
import { computed } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { createShellCommands } from "../commands";

function workspace(): WorkspaceDefinition {
  return {
    view: {} as never,
    sidebar: {} as never,
    inspector: {} as never,
    toolbarLeading: {} as never,
    toolbar: {} as never,
    inspectorLayout: { minSize: 300, maxSize: 480 },
    commands: computed(() => []),
  };
}

function commands(workspaceActive: boolean) {
  return createShellCommands(workspace(), workspaceActive, {
    actions: {
      openCommandPalette: vi.fn<() => void>(),
      openSettings: vi.fn<() => void>(),
      openShortcuts: vi.fn<() => void>(),
      toggleSidebar: vi.fn<() => void>(),
      toggleInspector: vi.fn<() => void>(),
    },
    labels: {
      settings: "settings",
      commandPalette: "palette",
      shortcuts: "shortcuts",
      toggleSidebar: "sidebar",
      toggleInspector: "inspector",
    },
  });
}

describe("shell commands", () => {
  it("exposes workspace view commands only on the Spaces route", () => {
    const settings = commands(false).map((command) => command.id);
    expect(settings).not.toContain("view.toggle-sidebar");
    expect(settings).not.toContain("view.toggle-inspector");

    const spaces = commands(true).map((command) => command.id);
    expect(spaces).toContain("view.toggle-sidebar");
    expect(spaces).toContain("view.toggle-inspector");
  });

  it("exposes command palette and keyboard shortcuts", () => {
    const appCommands = commands(true);
    const ids = appCommands.map((command) => command.id);
    expect(ids).toContain("shell.command-palette");
    expect(ids).toContain("shell.shortcuts");

    const shortcuts = appCommands.find((command) => command.id === "shell.shortcuts");
    expect(shortcuts?.shortcut).toBe("$mod+/");
    expect(shortcuts?.category).toBe("general");
  });
});
