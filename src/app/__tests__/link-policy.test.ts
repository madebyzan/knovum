import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  openUrl: vi.fn<(url: string) => Promise<void>>(),
  toastError: vi.fn<(message: string, options?: { description?: string }) => void>(),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: mocks.openUrl }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));

import { installLinkPolicy } from "../link-policy";

let dispose: () => void;

function link(href: string, parent: HTMLElement = document.body): HTMLAnchorElement {
  const anchor = document.createElement("a");
  anchor.href = href;
  anchor.textContent = "label";
  parent.append(anchor);
  return anchor;
}

function click(target: Element, init: MouseEventInit = {}): MouseEvent {
  const event = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0, ...init });
  target.dispatchEvent(event);
  return event;
}

beforeEach(() => {
  document.body.replaceChildren();
  mocks.openUrl.mockReset().mockResolvedValue();
  mocks.toastError.mockReset();
  dispose = installLinkPolicy();
});
afterEach(() => dispose());

describe("link policy", () => {
  it("opens an external link in the system and keeps the window on the app", () => {
    const event = click(link("https://example.com/docs"));
    expect(event.defaultPrevented).toBe(true);
    expect(mocks.openUrl).toHaveBeenCalledExactlyOnceWith("https://example.com/docs");
  });

  it("goes nowhere for a path, which would replace the app", () => {
    const event = click(link("/Users/zand/agent-workspaces/PPT摘要.md"));
    expect(event.defaultPrevented).toBe(true);
    expect(mocks.openUrl).not.toHaveBeenCalled();
  });

  it("covers the label inside a link and a middle click", () => {
    const anchor = link("mailto:a@example.com");
    const label = document.createElement("strong");
    anchor.replaceChildren(label);
    const event = new MouseEvent("auxclick", { bubbles: true, cancelable: true, button: 1 });
    label.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
    expect(mocks.openUrl).toHaveBeenCalledExactlyOnceWith("mailto:a@example.com");
  });

  it("leaves a link a component handled, or one inside editable text, to its owner", () => {
    const handled = link("https://example.com/handled");
    handled.addEventListener("click", (event) => event.preventDefault());
    click(handled);
    const editor = document.createElement("div");
    editor.contentEditable = "true";
    document.body.append(editor);
    const editable = link("https://example.com/edited", editor);
    const event = click(editable);
    expect(event.defaultPrevented).toBe(false);
    expect(mocks.openUrl).not.toHaveBeenCalled();
  });

  it("reports a link the system could not open", async () => {
    mocks.openUrl.mockRejectedValueOnce(new Error("no handler"));
    click(link("https://example.com/"));
    await vi.waitFor(() => expect(mocks.toastError).toHaveBeenCalledOnce());
    expect(mocks.toastError.mock.calls[0]?.[1]).toEqual({ description: "no handler" });
  });
});
