import { LogicalPosition, PhysicalPosition, Position } from "@tauri-apps/api/dpi";
import type {
  Menu,
  CheckMenuItemOptions,
  IconMenuItemOptions,
  MenuItemOptions,
  PredefinedMenuItemOptions,
  SubmenuOptions,
} from "@tauri-apps/api/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic } from "../errors/error-format";
import { createNativeContextMenu } from "./native-context-menu";

export type ContextMenuItem =
  | MenuItemOptions
  | CheckMenuItemOptions
  | IconMenuItemOptions
  | PredefinedMenuItemOptions
  | SubmenuOptions;

export type ContextMenuGroup = readonly ContextMenuItem[];

type PopupMenuPosition = Parameters<typeof Menu.prototype.popup>[0] | { x: number; y: number } | undefined;

export interface PopupContextMenuOptions {
  position?: PopupMenuPosition;
  window?: Parameters<typeof Menu.prototype.popup>[1] | undefined;
  scope?: string | undefined;
}

let activePopup: Awaited<ReturnType<typeof createNativeContextMenu>> | undefined;

const ACCELERATOR_BY_TOKEN: Readonly<Record<string, string>> = {
  $mod: "CmdOrCtrl",
  mod: "CmdOrCtrl",
  cmd: "CmdOrCtrl",
  ctrl: "Ctrl",
  shift: "Shift",
  alt: "Alt",
  option: "Alt",
  enter: "Enter",
  escape: "Escape",
  tab: "Tab",
  space: "Space",
  backspace: "Backspace",
  delete: "Delete",
};

export function normalizeAccelerator(shortcut?: string | undefined): string | undefined {
  if (!shortcut?.trim()) return undefined;
  return shortcut
    .trim()
    .split("+")
    .map(
      (p) =>
        ACCELERATOR_BY_TOKEN[p.trim().toLowerCase()] ??
        (p.trim().length === 1 ? p.trim().toUpperCase() : p.trim()),
    )
    .join("+");
}

function wrapAction(scope: string, action: (menuItemId: string) => void | Promise<void>) {
  return (menuItemId: string) => {
    void Promise.resolve()
      .then(() => action(menuItemId))
      .catch((err) => logError(`[${scope}] application menu action failed: ${formatErrorDiagnostic(err)}`));
  };
}

function processMenuItem(item: ContextMenuItem, scope: string): ContextMenuItem {
  if ("items" in item && Array.isArray(item.items)) {
    return { ...item, items: item.items.map((child) => processMenuItem(child as ContextMenuItem, scope)) };
  }
  const processed = { ...item };
  if ("accelerator" in item && item.accelerator) {
    const norm = normalizeAccelerator(item.accelerator);
    if (norm) (processed as MenuItemOptions).accelerator = norm;
    else delete (processed as MenuItemOptions).accelerator;
  }
  if ("action" in item && item.action) {
    (processed as MenuItemOptions).action = wrapAction(scope, item.action);
  }
  return processed;
}

const isMenuGroup = (e: ContextMenuItem | ContextMenuGroup): e is ContextMenuGroup => Array.isArray(e);

export function flattenMenuGroups(
  entries: readonly (ContextMenuItem | ContextMenuGroup)[],
  scope = "ContextMenu",
): ContextMenuItem[] {
  const menuEntries: ContextMenuItem[] = [];
  const separator: PredefinedMenuItemOptions = { item: "Separator" };
  for (const entry of entries) {
    if (isMenuGroup(entry)) {
      if (!entry.length) continue;
      if (menuEntries.length > 0) menuEntries.push(separator);
      for (const groupEntry of entry) menuEntries.push(processMenuItem(groupEntry, scope));
    } else {
      menuEntries.push(processMenuItem(entry, scope));
    }
  }
  return menuEntries;
}

function toMenuPosition(pos?: PopupMenuPosition): PhysicalPosition | LogicalPosition | Position | undefined {
  if (!pos) return undefined;
  if (pos instanceof LogicalPosition || pos instanceof PhysicalPosition || pos instanceof Position)
    return pos;
  return new LogicalPosition(pos.x, pos.y);
}

export async function popupContextMenu(
  items: readonly (ContextMenuItem | ContextMenuGroup)[],
  options: PopupContextMenuOptions = {},
): Promise<void> {
  const scope = options.scope ?? "ContextMenu";
  const processed = flattenMenuGroups(items, scope);
  if (!processed.length) return;

  const popupPos = toMenuPosition(options.position);
  let popup: typeof activePopup;
  try {
    popup = await createNativeContextMenu(processed);
    const previous = activePopup;
    activePopup = popup;
    await previous
      ?.close()
      .catch((err) => logError(`[${scope}] menu close failed: ${formatErrorDiagnostic(err)}`));
    // popup resolves on display; keep resources until the next popup replaces them.
    await popup.menu.popup(popupPos, options.window ?? getCurrentWindow());
  } catch (err) {
    if (popup) {
      if (activePopup === popup) activePopup = undefined;
      await popup
        .close()
        .catch((cause) => logError(`[${scope}] menu close failed: ${formatErrorDiagnostic(cause)}`));
    }
    void logError(`[${scope}] menu popup failed: ${formatErrorDiagnostic(err)}`);
  }
}

export function menuItem(
  text: string,
  action: () => void | Promise<void>,
  options: { enabled?: boolean; accelerator?: string; scope?: string } = {},
): MenuItemOptions {
  const item: MenuItemOptions = {
    text,
    enabled: options.enabled ?? true,
    action: wrapAction(options.scope ?? "MenuItem", action),
  };
  if (options.accelerator) {
    const norm = normalizeAccelerator(options.accelerator);
    if (norm) item.accelerator = norm;
  }
  return item;
}

export function checkMenuItem(
  text: string,
  checked: boolean,
  action: () => void | Promise<void>,
  options: { enabled?: boolean; scope?: string } = {},
): CheckMenuItemOptions {
  return {
    text,
    checked,
    enabled: options.enabled ?? true,
    action: wrapAction(options.scope ?? "CheckMenuItem", action),
  };
}
