import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { bindCommandShortcuts } from "../command-shortcuts";

const mocks = vi.hoisted(() => ({
  platform: vi.fn<() => string>(() => "macos"),
  logError: vi.fn<(message: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-os", () => ({ platform: mocks.platform }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));

const SPACE_PRESS = { key: " ", code: "Space" };
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

function command(shortcut: string, extra: Partial<AppCommand> = {}): AppCommand {
  return {
    id: `test.${shortcut.replaceAll(" ", "space")}`,
    title: shortcut,
    shortcut,
    run: vi.fn<() => void>(),
    ...extra,
  };
}

function press(target: Element, init: KeyboardEventInit) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

function mounted(tag: string, role?: string) {
  const element = document.createElement(tag);
  if (role) element.setAttribute("role", role);
  document.body.append(element);
  return element;
}

describe("application command shortcuts", () => {
  beforeEach(() => {
    mocks.platform.mockReturnValue("macos");
    document.body.replaceChildren();
  });

  it("binds the spacebar by key code and claims the press until it is unbound", async () => {
    const quickLook = command("Space");
    const unbind = bindCommandShortcuts([quickLook]);
    const event = press(mounted("div"), SPACE_PRESS);
    await settle();
    expect(quickLook.run).toHaveBeenCalledTimes(1);
    expect(event.defaultPrevented).toBe(true);

    unbind();
    press(mounted("div"), SPACE_PRESS);
    await settle();
    expect(quickLook.run).toHaveBeenCalledTimes(1);
  });

  it("cannot bind a literal space character, which tinykeys parses into an empty press", async () => {
    const unbindable = command(" ");
    const unbind = bindCommandShortcuts([unbindable]);
    press(mounted("div"), SPACE_PRESS);
    await settle();
    expect(unbindable.run).not.toHaveBeenCalled();
    unbind();
  });

  it("leaves a bare key to the focused control and keeps it for the canvas otherwise", async () => {
    const quickLook = command("Space");
    const unbind = bindCommandShortcuts([quickLook]);
    for (const target of [mounted("button"), mounted("a"), mounted("div", "menuitem")]) {
      expect(press(target, SPACE_PRESS).defaultPrevented).toBe(false);
    }
    await settle();
    expect(quickLook.run).not.toHaveBeenCalled();

    press(mounted("div"), SPACE_PRESS);
    await settle();
    expect(quickLook.run).toHaveBeenCalledTimes(1);
    unbind();
  });

  it("keeps a modifier chord pressed on a focused control", async () => {
    const selectAll = command("Shift+a");
    const unbind = bindCommandShortcuts([selectAll]);
    press(mounted("button"), { key: "A", code: "KeyA", shiftKey: true });
    await settle();
    expect(selectAll.run).toHaveBeenCalledTimes(1);
    unbind();
  });

  it("runs canvas clipboard chords on a focused canvas host", async () => {
    const paste = command("$mod+V");
    const unbind = bindCommandShortcuts([paste]);
    const macLike = /Mac|iPod|iPhone|iPad/.test(navigator.platform);
    const event = press(mounted("div"), {
      key: "v",
      code: "KeyV",
      metaKey: macLike,
      ctrlKey: !macLike,
    });
    await settle();
    expect(paste.run).toHaveBeenCalledOnce();
    expect(event.defaultPrevented).toBe(true);
    unbind();
  });

  it("honours allowInEditable in a text field and leaves other chords to the field", async () => {
    const palette = command("Meta+K", { allowInEditable: true });
    const canvasOnly = command("Meta+J");
    const unbind = bindCommandShortcuts([palette, canvasOnly]);
    const field = mounted("input");
    const allowed = press(field, { key: "K", code: "KeyK", metaKey: true });
    const withheld = press(field, { key: "J", code: "KeyJ", metaKey: true });
    await settle();
    expect(palette.run).toHaveBeenCalledTimes(1);
    expect(allowed.defaultPrevented).toBe(true);
    expect(canvasOnly.run).not.toHaveBeenCalled();
    expect(withheld.defaultPrevented).toBe(false);
    unbind();
  });

  it("preserves a native text-editing chord in an editable field", async () => {
    const copy = command("Meta+C");
    const unbind = bindCommandShortcuts([copy]);

    const event = press(mounted("input"), { key: "c", code: "KeyC", metaKey: true });
    await settle();

    expect(copy.run).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
    unbind();
  });

  it("leaves autorepeat and IME composition to the focused control", async () => {
    const palette = command("Meta+K", { allowInEditable: true });
    const unbind = bindCommandShortcuts([palette]);
    const field = mounted("input");
    press(field, { key: "K", code: "KeyK", metaKey: true, repeat: true });
    press(field, { key: "K", code: "KeyK", metaKey: true, isComposing: true });
    await settle();
    expect(palette.run).not.toHaveBeenCalled();
    unbind();
  });

  it("leaves a shortcut the macOS Application Menu owns to that menu", async () => {
    const menuOwned = command("Shift+=", { appMenuShortcut: true });
    const keyboardOwned = command("Shift+0");
    const unbind = bindCommandShortcuts([menuOwned, keyboardOwned]);
    press(mounted("div"), { key: "=", code: "Equal", shiftKey: true });
    press(mounted("div"), { key: "0", code: "Digit0", shiftKey: true });
    await settle();
    expect(menuOwned.run).not.toHaveBeenCalled();
    expect(keyboardOwned.run).toHaveBeenCalledTimes(1);
    unbind();
  });

  it("binds a menu-owned shortcut where there is no Application Menu", async () => {
    mocks.platform.mockReturnValue("windows");
    const menuOwned = command("Shift+=");
    const unbind = bindCommandShortcuts([menuOwned]);
    press(mounted("div"), { key: "=", code: "Equal", shiftKey: true });
    await settle();
    expect(menuOwned.run).toHaveBeenCalledTimes(1);
    unbind();
  });

  it("supports desktop canvas cancel and scene navigation keys", async () => {
    const cancel = command("Escape");
    const previousScene = command("Alt+[");
    const unbind = bindCommandShortcuts([cancel, previousScene]);

    press(mounted("div"), { key: "Escape", code: "Escape" });
    press(mounted("div"), { key: "[", code: "BracketLeft", altKey: true });
    await settle();

    expect(cancel.run).toHaveBeenCalledOnce();
    expect(previousScene.run).toHaveBeenCalledOnce();
    unbind();
  });

  it("uses physical keybindings while preserving the displayed shortcut", async () => {
    const zoomToFit = command("Shift+1", { keybindings: ["Shift+Digit1"] });
    const unbind = bindCommandShortcuts([zoomToFit]);

    press(mounted("div"), { key: "!", code: "Digit1", shiftKey: true });
    await settle();

    expect(zoomToFit.run).toHaveBeenCalledOnce();
    unbind();
  });
});
