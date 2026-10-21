import { vi } from "vitest";

import type { AppCommand } from "../../../../shared/contracts/app-command";
import type { EditingCommand } from "../../../../shared/contracts/generated/EditingCommand";
import { registerAppCommands } from "../../run-app-command";

export type SpyCommand = AppCommand & { run: ReturnType<typeof vi.fn> };

export const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

export function surfaceCommand(id: string, editingIntent: EditingCommand, shortcut?: string): SpyCommand {
  return { id, title: id, editingIntent, ...(shortcut ? { shortcut } : {}), run: vi.fn<() => void>() };
}

export function canvasCommands() {
  return {
    undo: surfaceCommand("spaces.undo", "undo", "$mod+Z"),
    redo: surfaceCommand("spaces.redo", "redo", "$mod+Shift+Z"),
    cut: surfaceCommand("spaces.cut", "cut", "$mod+X"),
    copy: surfaceCommand("spaces.copy", "copy", "$mod+C"),
    paste: surfaceCommand("spaces.paste", "paste", "$mod+V"),
    selectAll: surfaceCommand("spaces.select-all", "select-all", "$mod+A"),
  };
}

export function stubHistory() {
  return { undo: vi.fn<() => void>(), redo: vi.fn<() => void>() };
}

export function html(markup: string) {
  document.body.insertAdjacentHTML("beforeend", markup);
}

export function byId(id: string): HTMLElement {
  return document.getElementById(id)!;
}

export function selectText(node: Node) {
  const range = document.createRange();
  range.selectNodeContents(node);
  const selection = document.getSelection()!;
  selection.removeAllRanges();
  selection.addRange(range);
}

export function resetPage() {
  document.getSelection()?.removeAllRanges();
  (document.activeElement as HTMLElement | null)?.blur?.();
  document.body.replaceChildren();
  registerAppCommands([]);
}
