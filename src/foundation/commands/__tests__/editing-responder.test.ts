import { beforeEach, describe, expect, it, vi } from "vitest";

import { registerEditorHistory, runEditingCommand } from "../editing-responder";
import { registerAppCommands } from "../run-app-command";
import {
  byId,
  canvasCommands,
  html,
  resetPage,
  selectText,
  settle,
  stubHistory,
  surfaceCommand,
} from "./support/editing-fixtures";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args: { command: string }) => Promise<boolean>>(() =>
    Promise.resolve(true),
  ),
  logError: vi.fn<(message: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));

function platformEdits() {
  return mocks.invoke.mock.calls.map(([, args]) => args.command);
}

beforeEach(() => {
  vi.clearAllMocks();
  resetPage();
});

describe("editing responder chain from the macOS Application Menu", () => {
  it("lets the platform edit a focused text field, history included", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<input id="field" />`);
    byId("field").focus();

    await runEditingCommand("copy");
    await runEditingCommand("undo");
    await settle();

    expect(platformEdits()).toEqual(["copy", "undo"]);
    for (const command of Object.values(canvas)) expect(command.run).not.toHaveBeenCalled();
  });

  it("copies selected read-only text instead of the canvas selection", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<p id="bubble">A chat reply worth keeping</p>`);
    selectText(byId("bubble"));

    await runEditingCommand("copy");
    await runEditingCommand("cut");
    await settle();

    expect(platformEdits()).toEqual(["copy", "cut"]);
    expect(canvas.copy.run).not.toHaveBeenCalled();
    expect(canvas.cut.run).not.toHaveBeenCalled();
  });

  it("keeps the canvas clipboard while the canvas has focus, despite a selection left elsewhere", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<p id="bubble">Earlier selection</p><div id="canvas" tabindex="0" data-editing-surface></div>`);
    selectText(byId("bubble"));
    byId("canvas").focus();

    await runEditingCommand("copy");
    await settle();

    expect(canvas.copy.run).toHaveBeenCalledOnce();
    expect(platformEdits()).toEqual([]);
  });

  it("copies text selected inside the focused canvas itself", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<div id="canvas" tabindex="0" data-editing-surface><p id="card">Card text</p></div>`);
    byId("canvas").focus();
    selectText(byId("card"));

    await runEditingCommand("copy");
    await settle();

    expect(platformEdits()).toEqual(["copy"]);
    expect(canvas.copy.run).not.toHaveBeenCalled();
  });

  it("leaves paste and select-all with the surface when only read-only text is selected", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<p id="bubble">Selected</p>`);
    selectText(byId("bubble"));

    await runEditingCommand("paste");
    await runEditingCommand("select-all");
    await settle();

    expect(canvas.paste.run).toHaveBeenCalledOnce();
    expect(canvas.selectAll.run).toHaveBeenCalledOnce();
    expect(platformEdits()).toEqual([]);
  });

  it("undoes rich text through its own history, never the canvas history", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<div id="note" contenteditable="true"><p id="line">Typing</p></div>`);
    const history = stubHistory();
    const release = registerEditorHistory(byId("note"), history);
    byId("note").focus();

    await runEditingCommand("undo");
    await runEditingCommand("redo");
    await runEditingCommand("paste");
    await settle();

    expect(history.undo).toHaveBeenCalledOnce();
    expect(history.redo).toHaveBeenCalledOnce();
    expect(canvas.undo.run).not.toHaveBeenCalled();
    expect(canvas.redo.run).not.toHaveBeenCalled();
    expect(platformEdits()).toEqual(["paste"]);
    release();
  });

  it("gives undo to the platform for rich text that keeps no history of its own", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(`<div id="plain" contenteditable="true"></div>`);
    byId("plain").focus();

    await runEditingCommand("undo");
    await settle();

    expect(platformEdits()).toEqual(["undo"]);
    expect(canvas.undo.run).not.toHaveBeenCalled();
  });

  it("reaches any surface that declares the intent, such as the video editor", async () => {
    const videoUndo = surfaceCommand("video-edit.undo", "undo");
    registerAppCommands([videoUndo]);

    await runEditingCommand("undo");
    await settle();

    expect(videoUndo.run).toHaveBeenCalledOnce();
  });

  it("does nothing when no responder answers, rather than editing the page itself", async () => {
    registerAppCommands([]);

    await runEditingCommand("select-all");
    await runEditingCommand("undo");
    await settle();

    expect(platformEdits()).toEqual([]);
  });

  it("copies read-only text selected in a modal, while the surface behind it stays out of reach", async () => {
    const canvas = canvasCommands();
    registerAppCommands(Object.values(canvas));
    html(
      `<div data-app-command-scope="" data-state="open"><p id="message">Error details</p><button id="ok">OK</button></div>`,
    );
    selectText(byId("message"));

    await runEditingCommand("copy");
    document.getSelection()?.removeAllRanges();
    await runEditingCommand("copy");
    await settle();

    expect(platformEdits()).toEqual(["copy"]);
    expect(canvas.copy.run).not.toHaveBeenCalled();
  });
});
