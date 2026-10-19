import { beforeEach, describe, expect, it, vi } from "vitest";

import { registerAppCommands } from "../../../foundation/commands/run-app-command";
import type { AppCommand } from "../../../shared/contracts/app-command";
import { runEditingCommand } from "../editing-command";

const editing = vi.hoisted(() => ({ native: vi.fn<(command: string) => void>() }));
vi.mock("../../../foundation/commands/text-editing", () => ({
  performApplicationMenuEditing: editing.native,
}));

function surfaceCommand(id: string): AppCommand & { run: ReturnType<typeof vi.fn> } {
  return { id, title: id, run: vi.fn<() => void>() };
}

function focus(element: HTMLElement) {
  document.body.append(element);
  element.focus();
}

beforeEach(() => {
  vi.clearAllMocks();
  document.body.innerHTML = "";
});

describe("application menu editing", () => {
  it("lets the platform edit the focused text control", async () => {
    const canvasCopy = surfaceCommand("spaces.copy");
    registerAppCommands([canvasCopy]);
    const field = document.createElement("input");
    focus(field);

    await runEditingCommand("copy");

    expect(editing.native).toHaveBeenCalledWith("copy");
    expect(canvasCopy.run).not.toHaveBeenCalled();
  });

  it("hands clipboard and selection to the active surface outside a text control", async () => {
    const commands = ["spaces.cut", "spaces.copy", "spaces.paste", "spaces.select-all"].map(surfaceCommand);
    registerAppCommands(commands);

    await Promise.all(
      (["cut", "copy", "paste", "select-all"] as const).map((intent) => runEditingCommand(intent)),
    );
    await vi.waitFor(() => {
      for (const command of commands) expect(command.run).toHaveBeenCalledOnce();
    });
    expect(editing.native).not.toHaveBeenCalled();
  });

  it("keeps undo with the surface that owns the history, even inside a text control", async () => {
    const documentUndo = surfaceCommand("document.undo");
    const canvasUndo = surfaceCommand("spaces.undo");
    registerAppCommands([documentUndo, canvasUndo]);
    focus(document.createElement("textarea"));

    await runEditingCommand("undo");

    await vi.waitFor(() => expect(documentUndo.run).toHaveBeenCalledOnce());
    expect(canvasUndo.run).not.toHaveBeenCalled();
    expect(editing.native).not.toHaveBeenCalled();
  });

  it("falls back to the canvas history when no document is open", async () => {
    const canvasRedo = surfaceCommand("spaces.redo");
    registerAppCommands([canvasRedo]);

    await runEditingCommand("redo");

    await vi.waitFor(() => expect(canvasRedo.run).toHaveBeenCalledOnce());
  });
});
