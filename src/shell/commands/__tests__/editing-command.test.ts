import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { registerAppCommands } from "../../../foundation/commands/run-app-command";
import type { AppCommand } from "../../../shared/contracts/app-command";
import { runEditingCommand } from "../editing-command";

const editing = vi.hoisted(() => ({ applicationMenu: vi.fn<(command: string) => void>() }));
vi.mock("../../../foundation/commands/text-editing", () => ({
  performApplicationMenuEditing: editing.applicationMenu,
}));

function surfaceCommand(id: string): AppCommand & { run: ReturnType<typeof vi.fn> } {
  return { id, title: id, run: vi.fn<() => void>() };
}

function focus(markup: string) {
  const wrapper = mount({ template: `<div>${markup}</div>` }, { attachTo: document.body });
  wrapper.get<HTMLElement>("[data-testid='target']").element.focus();
  return wrapper;
}

beforeEach(() => {
  vi.clearAllMocks();
  document.body.innerHTML = "";
});

describe("application menu editing", () => {
  it("lets the platform edit the focused text control", async () => {
    const canvasCopy = surfaceCommand("spaces.copy");
    const canvasUndo = surfaceCommand("spaces.undo");
    registerAppCommands([canvasCopy, canvasUndo]);
    focus(`<input data-testid="target" />`);

    await runEditingCommand("copy");
    await runEditingCommand("undo");

    expect(editing.applicationMenu.mock.calls).toEqual([["copy"], ["undo"]]);
    expect(canvasCopy.run).not.toHaveBeenCalled();
    expect(canvasUndo.run).not.toHaveBeenCalled();
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
    expect(editing.applicationMenu).not.toHaveBeenCalled();
  });

  it("undoes a rich-text surface through its own history, not the platform stack", async () => {
    const documentUndo = surfaceCommand("document.undo");
    const canvasUndo = surfaceCommand("spaces.undo");
    registerAppCommands([documentUndo, canvasUndo]);
    focus(`<div data-testid="target" contenteditable="true"></div>`);

    await runEditingCommand("undo");

    await vi.waitFor(() => expect(documentUndo.run).toHaveBeenCalledOnce());
    expect(canvasUndo.run).not.toHaveBeenCalled();
    expect(editing.applicationMenu).not.toHaveBeenCalled();
  });

  it("falls back to the canvas history when no document is open", async () => {
    const canvasRedo = surfaceCommand("spaces.redo");
    registerAppCommands([canvasRedo]);

    await runEditingCommand("redo");

    await vi.waitFor(() => expect(canvasRedo.run).toHaveBeenCalledOnce());
  });

  it("gives the clipboard to a rich-text surface so its own paste handling runs", async () => {
    const canvasPaste = surfaceCommand("spaces.paste");
    registerAppCommands([canvasPaste]);
    focus(`<div data-testid="target" contenteditable="true"></div>`);

    await runEditingCommand("paste");

    expect(editing.applicationMenu).toHaveBeenCalledWith("paste");
    expect(canvasPaste.run).not.toHaveBeenCalled();
  });
});
