import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { bindCommandShortcuts } from "../command-shortcuts";

const mocks = vi.hoisted(() => ({
  platform: vi.fn<() => string>(() => "macos"),
  logError: vi.fn<(message: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-os", () => ({ platform: mocks.platform }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

function command(shortcut: string, extra: Partial<AppCommand> = {}): AppCommand {
  return { id: `test.${shortcut}`, title: shortcut, shortcut, run: vi.fn<() => void>(), ...extra };
}

function press(target: Element, init: KeyboardEventInit) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

function mounted(tag: string, attributes: Record<string, string> = {}) {
  const element = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) element.setAttribute(name, value);
  document.body.append(element);
  return element;
}

describe("platform text editing chords", () => {
  beforeEach(() => {
    mocks.platform.mockReturnValue("windows");
    document.body.replaceChildren();
  });

  it("leaves text editing to the text being edited, even for a command allowed while typing", async () => {
    const documentUndo = command("$mod+Z", { allowInEditable: true });
    const unbind = bindCommandShortcuts([documentUndo]);
    const macLike = /Mac|iPod|iPhone|iPad/.test(navigator.platform);
    const chord = { key: "z", code: "KeyZ", metaKey: macLike, ctrlKey: !macLike };

    const field = mounted("input");
    const richText = mounted("div", { contenteditable: "true" });
    const inField = press(field, chord);
    const inRichText = press(richText, chord);
    await settle();

    expect(documentUndo.run).not.toHaveBeenCalled();
    expect(inField.defaultPrevented).toBe(false);
    expect(inRichText.defaultPrevented).toBe(false);

    press(mounted("div"), chord);
    await settle();
    expect(documentUndo.run).toHaveBeenCalledOnce();
    unbind();
  });
});
