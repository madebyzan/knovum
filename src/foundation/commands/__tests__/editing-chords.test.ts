import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { bindCommandShortcuts } from "../command-shortcuts";
import { registerEditorHistory } from "../editing-responder";
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

// Windows and Linux: the WebView receives the editing chords, so the shortcut layer resolves them.
vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "windows" }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));

const macLike = /Mac|iPod|iPhone|iPad/.test(navigator.platform);

function chord(key: string, extra: KeyboardEventInit = {}): KeyboardEventInit {
  return { key, code: `Key${key.toUpperCase()}`, metaKey: macLike, ctrlKey: !macLike, ...extra };
}

function press(target: EventTarget, init: KeyboardEventInit) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

beforeEach(resetPage);

describe("editing responder chain from the keyboard", () => {
  it("lets the platform copy selected read-only text", async () => {
    const canvas = canvasCommands();
    const unbind = bindCommandShortcuts(Object.values(canvas));
    html(`<p id="bubble">A chat reply worth keeping</p>`);
    selectText(byId("bubble"));

    const event = press(document.body, chord("c"));
    await settle();

    expect(event.defaultPrevented).toBe(false);
    expect(canvas.copy.run).not.toHaveBeenCalled();
    unbind();
  });

  it("runs the surface command when nothing on the page is being edited", async () => {
    const canvas = canvasCommands();
    const unbind = bindCommandShortcuts(Object.values(canvas));

    const paste = press(document.body, chord("v"));
    const redo = press(document.body, chord("y"));
    await settle();

    expect(paste.defaultPrevented).toBe(true);
    expect(canvas.paste.run).toHaveBeenCalledOnce();
    expect(redo.defaultPrevented).toBe(true);
    expect(canvas.redo.run).toHaveBeenCalledOnce();
    unbind();
  });

  it("leaves undo to a rich-text editor's own keymap", async () => {
    const canvas = canvasCommands();
    const unbind = bindCommandShortcuts(Object.values(canvas));
    html(`<div id="note" contenteditable="true"></div>`);
    const release = registerEditorHistory(byId("note"), stubHistory());

    const event = press(byId("note"), chord("z"));
    await settle();

    expect(event.defaultPrevented).toBe(false);
    expect(canvas.undo.run).not.toHaveBeenCalled();
    release();
    unbind();
  });

  it("claims an unanswered chord so the page is never selected or undone as a whole", async () => {
    const unbind = bindCommandShortcuts([]);

    const selectAll = press(document.body, chord("a"));

    expect(selectAll.defaultPrevented).toBe(true);
    unbind();
  });

  it("repeats nothing while a chord is held, but still keeps the press from the page", async () => {
    const canvas = canvasCommands();
    const unbind = bindCommandShortcuts(Object.values(canvas));

    const held = press(document.body, chord("z", { repeat: true }));
    await settle();

    expect(held.defaultPrevented).toBe(true);
    expect(canvas.undo.run).not.toHaveBeenCalled();
    unbind();
  });
});

describe("editing command declarations", () => {
  it("refuses a command that binds an editing chord without declaring the intent", () => {
    const rogue: AppCommand = {
      id: "rogue.copy",
      title: "Copy",
      shortcut: "$mod+C",
      run: vi.fn<() => void>(),
    };
    expect(() => bindCommandShortcuts([rogue])).toThrow(/editing chord \$mod\+C/);
  });

  it("refuses two responders for one intent on the same surface", () => {
    const first = surfaceCommand("document.undo", "undo");
    const second = surfaceCommand("spaces.undo", "undo");
    expect(() => bindCommandShortcuts([first, second])).toThrow(/second responder for "undo"/);
  });
});
