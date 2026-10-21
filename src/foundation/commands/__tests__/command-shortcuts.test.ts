import { beforeEach, describe, expect, it, vi } from "vitest";

import { bindCommandShortcuts } from "../command-shortcuts";
import { command, mounted, press, settle, shortcutHost } from "./support/shortcut-harness";

const harness = vi.hoisted(() => () => import("./support/shortcut-harness"));
vi.mock("@tauri-apps/plugin-os", async () => (await harness()).osPlugin);
vi.mock("@tauri-apps/plugin-log", async () => (await harness()).logPlugin);

const SPACE_PRESS = { key: " ", code: "Space" };

describe("application command shortcuts", () => {
  beforeEach(() => {
    shortcutHost.platform.mockReturnValue("macos");
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
    for (const target of [mounted("button"), mounted("a"), mounted("div", { role: "menuitem" })]) {
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

  it("runs a surface clipboard command on a focused canvas host", async () => {
    shortcutHost.platform.mockReturnValue("windows");
    const paste = command("$mod+V", { editingIntent: "paste" });
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
    shortcutHost.platform.mockReturnValue("windows");
    const copy = command("$mod+C", { editingIntent: "copy" });
    const unbind = bindCommandShortcuts([copy]);
    const macLike = /Mac|iPod|iPhone|iPad/.test(navigator.platform);

    const event = press(mounted("input"), { key: "c", code: "KeyC", metaKey: macLike, ctrlKey: !macLike });
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
    shortcutHost.platform.mockReturnValue("windows");
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
