import { beforeEach, describe, expect, it, vi } from "vitest";

import { createCanvasEditingCommands } from "../canvas-editing-commands";

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("@tauri-apps/plugin-log", () => ({
  info: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
}));

const copySelection = vi.fn<() => Promise<void>>();
const pasteSelection = vi.fn<() => Promise<boolean>>();
const editor = {
  copySelection,
  cutSelection: vi.fn<() => Promise<void>>(),
  deleteSelection: vi.fn<() => Promise<void>>(),
  duplicateSelection: vi.fn<() => Promise<void>>(),
  pasteSelection,
  selectAll: vi.fn<() => void>(),
};

describe("canvas editing commands", () => {
  beforeEach(() => vi.clearAllMocks());

  it("keeps clipboard commands discoverable and disables selection-only actions", async () => {
    const commands = createCanvasEditingCommands(options(false));
    const copy = command(commands, "spaces.copy");
    const paste = command(commands, "spaces.paste");

    expect(copy.shortcut).toBe("$mod+C");
    expect(copy.appMenuShortcut).toBe(true);
    expect(copy.allowInEditable).toBeUndefined();
    expect(copy.availability?.()).toEqual({ state: "blocked", reason: "no_selection" });
    expect(paste.shortcut).toBe("$mod+V");
    await paste.run();
    expect(pasteSelection).toHaveBeenCalledOnce();
  });

  it("routes copy through the neutral editor port", async () => {
    const copy = command(createCanvasEditingCommands(options(true)), "spaces.copy");
    expect(copy.availability?.()).toEqual({ state: "ready" });
    await copy.run();
    expect(copySelection).toHaveBeenCalledOnce();
  });
});

function options(hasSelection: boolean): Parameters<typeof createCanvasEditingCommands>[0] {
  return {
    canRedo: () => false,
    canUndo: () => false,
    editor: () => editor,
    hasNodeSelection: () => hasSelection,
    hasSelection: () => hasSelection,
    labels: {
      copy: "Copy",
      cut: "Cut",
      deleteSelection: "Delete",
      duplicate: "Duplicate",
      paste: "Paste",
      redo: "Redo",
      selectAll: "Select All",
      undo: "Undo",
    },
    redo: vi.fn<() => Promise<void>>(),
    undo: vi.fn<() => Promise<void>>(),
  };
}

function command(commands: ReturnType<typeof createCanvasEditingCommands>, commandId: string) {
  const selectedCommand = commands.find((candidate) => candidate.id === commandId);
  if (!selectedCommand) throw new Error(`Missing command ${commandId}`);
  return selectedCommand;
}
