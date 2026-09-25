import { describe, expect, it, vi } from "vitest";
import { computed } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { createShellCommands } from "../commands";

function workspace(rightSidebarAvailable = true): WorkspaceDefinition {
  return {
    view: {} as never,
    sidebar: {} as never,
    rightSidebar: {} as never,
    rightSidebarToolbar: {} as never,
    toolbarLeading: {} as never,
    toolbar: {} as never,
    rightSidebarLayout: { defaultSize: 380, minSize: 300, maxSize: 480 },
    rightSidebarAvailable: computed(() => rightSidebarAvailable),
    openDefaultRightSidebar: vi.fn<() => void>(),
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
      toggleRightSidebar: vi.fn<() => void>(),
    },
    labels: {
      settings: "settings",
      commandPalette: "palette",
      shortcuts: "shortcuts",
      toggleSidebar: "sidebar",
      toggleRightSidebar: "right sidebar",
    },
  });
}

describe("shell commands", () => {
  it("exposes workspace view commands only on the Spaces route", () => {
    const settings = commands(false).map((command) => command.id);
    expect(settings).not.toContain("view.toggle-sidebar");
    expect(settings).not.toContain("view.toggle-right-sidebar");

    const spaces = commands(true).map((command) => command.id);
    expect(spaces).toContain("view.toggle-sidebar");
    expect(spaces).toContain("view.toggle-right-sidebar");
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

  it("omits the right sidebar command when the current workspace view cannot show it", () => {
    const ids = createShellCommands(workspace(false), true, {
      actions: {
        openCommandPalette: vi.fn<() => void>(),
        openSettings: vi.fn<() => void>(),
        toggleSidebar: vi.fn<() => void>(),
        toggleRightSidebar: vi.fn<() => void>(),
      },
      labels: {
        settings: "settings",
        commandPalette: "palette",
        toggleSidebar: "sidebar",
        toggleRightSidebar: "right sidebar",
      },
    }).map((command) => command.id);
    expect(ids).not.toContain("view.toggle-right-sidebar");
  });
});
