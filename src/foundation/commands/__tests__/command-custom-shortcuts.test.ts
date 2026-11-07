import { beforeEach, describe, expect, it, vi } from "vitest";

import { bindCommandShortcuts } from "../command-shortcuts";
import { command, mounted, press, settle, shortcutHost } from "./support/shortcut-harness";

const harness = vi.hoisted(() => () => import("./support/shortcut-harness"));
vi.mock("@tauri-apps/plugin-os", async () => (await harness()).osPlugin);
vi.mock("@tauri-apps/plugin-log", async () => (await harness()).logPlugin);

function resolveSelectTool(id: string, defaultShortcut?: string): string | undefined {
  return id === "spaces.tool-select" ? "1" : defaultShortcut;
}

function resolveZoomIn(id: string, defaultShortcut?: string): string | undefined {
  return id === "spaces.zoom-in" ? "Alt+8" : defaultShortcut;
}

function resolveNewTab(id: string, defaultShortcut?: string): string | undefined {
  return id === "spaces.new-space" ? "Alt+N" : defaultShortcut;
}

describe("customizable command shortcuts", () => {
  beforeEach(() => {
    shortcutHost.platform.mockReturnValue("macos");
    document.body.replaceChildren();
  });

  it("binds customized shortcut and runs command on custom chord", async () => {
    const selectTool = command("V", { id: "spaces.tool-select" });

    const unbind = bindCommandShortcuts([selectTool], resolveSelectTool);

    // Old default chord "V" should not run the command
    press(mounted("div"), { key: "v", code: "KeyV" });
    await settle();
    expect(selectTool.run).not.toHaveBeenCalled();

    // New custom chord "1" should run the command
    press(mounted("div"), { key: "1", code: "Digit1" });
    await settle();
    expect(selectTool.run).toHaveBeenCalledOnce();

    unbind();
  });

  it("customized shortcut overrides default physical keybindings", async () => {
    const zoomIn = command("$mod+=", {
      id: "spaces.zoom-in",
      keybindings: ["$mod+Equal", "$mod+Shift+Equal"],
    });

    const unbind = bindCommandShortcuts([zoomIn], resolveZoomIn);

    // Default keybinding should not run
    press(mounted("div"), { key: "=", code: "Equal", metaKey: true });
    await settle();
    expect(zoomIn.run).not.toHaveBeenCalled();

    // Custom shortcut Alt+8 runs
    press(mounted("div"), { key: "8", code: "Digit8", altKey: true });
    await settle();
    expect(zoomIn.run).toHaveBeenCalledOnce();

    unbind();
  });

  it("binds customized menu-owned command on macOS in webview", async () => {
    shortcutHost.platform.mockReturnValue("macos");
    const newTab = command("$mod+T", { id: "spaces.new-space", appMenuShortcut: true });

    const unbind = bindCommandShortcuts([newTab], resolveNewTab);

    // Custom shortcut Alt+N triggers the command
    press(mounted("div"), { key: "n", code: "KeyN", altKey: true });
    await settle();
    expect(newTab.run).toHaveBeenCalledOnce();

    unbind();
  });

  it("restores default shortcut after reset", async () => {
    const handTool = command("H", { id: "spaces.tool-pan" });
    let isCustomized = true;
    const resolver = (id: string, defaultShortcut?: string) =>
      isCustomized && id === "spaces.tool-pan" ? "2" : defaultShortcut;

    // Custom phase
    let unbind = bindCommandShortcuts([handTool], resolver);
    press(mounted("div"), { key: "2", code: "Digit2" });
    await settle();
    expect(handTool.run).toHaveBeenCalledTimes(1);
    unbind();

    // Reset phase
    isCustomized = false;
    unbind = bindCommandShortcuts([handTool], resolver);

    // Custom chord "2" no longer works
    press(mounted("div"), { key: "2", code: "Digit2" });
    await settle();
    expect(handTool.run).toHaveBeenCalledTimes(1);

    // Default chord "H" works again
    press(mounted("div"), { key: "h", code: "KeyH" });
    await settle();
    expect(handTool.run).toHaveBeenCalledTimes(2);

    unbind();
  });
});
