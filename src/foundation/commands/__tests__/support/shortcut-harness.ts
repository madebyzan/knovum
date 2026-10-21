import { vi } from "vitest";

import type { AppCommand } from "../../../../shared/contracts/app-command";

/** The platform plugins a shortcut binding reads; suites route their `vi.mock` factories here. */
export const shortcutHost = {
  platform: vi.fn<() => string>(() => "macos"),
  logError: vi.fn<(message: string) => Promise<void>>(),
};
export const osPlugin = { platform: shortcutHost.platform };
export const logPlugin = { error: shortcutHost.logError };

export const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

export function command(shortcut: string, extra: Partial<AppCommand> = {}): AppCommand {
  return {
    id: `test.${shortcut.replaceAll(" ", "space")}`,
    title: shortcut,
    shortcut,
    run: vi.fn<() => void>(),
    ...extra,
  };
}

export function press(target: Element, init: KeyboardEventInit) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

export function mounted(tag: string, attributes: Record<string, string> = {}) {
  const element = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) element.setAttribute(name, value);
  document.body.append(element);
  return element;
}
