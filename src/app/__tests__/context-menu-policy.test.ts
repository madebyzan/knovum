import { afterEach, describe, expect, it, vi } from "vitest";

import { installContextMenuPolicy } from "../context-menu-policy";

let dispose: (() => void) | undefined;

afterEach(() => {
  dispose?.();
  dispose = undefined;
  document.body.replaceChildren();
  vi.unstubAllEnvs();
});

function installProductionPolicy() {
  vi.stubEnv("DEV", false);
  dispose = installContextMenuPolicy();
}

function rightClick(target: Element) {
  const event = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event;
}

describe("application context menu policy", () => {
  it("keeps the default menu available in development", () => {
    vi.stubEnv("DEV", true);
    dispose = installContextMenuPolicy();

    expect(rightClick(document.body).defaultPrevented).toBe(false);
  });

  it("suppresses the browser menu on production workbench surfaces", () => {
    installProductionPolicy();

    expect(rightClick(document.body).defaultPrevented).toBe(true);
  });

  it.each(["input", "textarea", "select"])("preserves native editing menus on %s", (tag) => {
    installProductionPolicy();
    const input = document.createElement(tag);
    document.body.append(input);

    expect(rightClick(input).defaultPrevented).toBe(false);
  });

  it("preserves native editing menus inside rich-text editors", () => {
    installProductionPolicy();
    const editor = document.createElement("div");
    editor.setAttribute("contenteditable", "true");
    const text = document.createElement("span");
    editor.append(text);
    document.body.append(editor);

    expect(rightClick(text).defaultPrevented).toBe(false);
  });

  it("allows product menu handlers to run even when they stop propagation", () => {
    installProductionPolicy();
    const target = document.createElement("div");
    const showProductMenu = vi.fn<(event: Event) => void>((event) => event.stopPropagation());
    target.addEventListener("contextmenu", showProductMenu);
    document.body.append(target);

    expect(rightClick(target).defaultPrevented).toBe(true);
    expect(showProductMenu).toHaveBeenCalledOnce();
  });

  it("covers menus in overlays mounted outside the Vue application root", () => {
    installProductionPolicy();
    const overlay = document.createElement("div");
    document.body.append(overlay);

    expect(rightClick(overlay).defaultPrevented).toBe(true);
  });

  it("removes the document policy when disposed", () => {
    installProductionPolicy();
    dispose?.();
    dispose = undefined;

    expect(rightClick(document.body).defaultPrevented).toBe(false);
  });
});
