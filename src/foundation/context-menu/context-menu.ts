import { LogicalPosition, PhysicalPosition, Position } from "@tauri-apps/api/dpi";
import {
  Menu,
  type CheckMenuItemOptions,
  type IconMenuItemOptions,
  type MenuItemOptions,
  type PredefinedMenuItemOptions,
  type SubmenuOptions,
} from "@tauri-apps/api/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";

export type { CheckMenuItemOptions, MenuItemOptions, PredefinedMenuItemOptions, SubmenuOptions };

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

const ACCELERATOR_MAP: Readonly<Record<string, string>> = {
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
        ACCELERATOR_MAP[p.trim().toLowerCase()] ??
        (p.trim().length === 1 ? p.trim().toUpperCase() : p.trim()),
    )
    .join("+");
}

function wrapAction(scope: string, action?: ((id: string) => void | Promise<void>) | undefined) {
  if (!action) return undefined;
  return (id: string) => {
    void Promise.resolve()
      .then(() => action(id))
      .catch((err) => logError(`[${scope}] application menu action failed: ${String(err)}`));
  };
}

function processMenuItem(item: ContextMenuItem, scope: string): ContextMenuItem {
  if ("items" in item && Array.isArray(item.items)) {
    return { ...item, items: item.items.map((sub) => processMenuItem(sub as ContextMenuItem, scope)) };
  }
  const processed = { ...item };
  if ("accelerator" in item && item.accelerator) {
    const norm = normalizeAccelerator(item.accelerator);
    if (norm) (processed as MenuItemOptions).accelerator = norm;
    else delete (processed as MenuItemOptions).accelerator;
  }
  if ("action" in item && item.action) {
    const wrapped = wrapAction(scope, item.action);
    if (wrapped) (processed as MenuItemOptions).action = wrapped;
    else delete (processed as MenuItemOptions).action;
  }
  return processed;
}

const isMenuGroup = (e: ContextMenuItem | ContextMenuGroup): e is ContextMenuGroup => Array.isArray(e);

export function flattenMenuGroups(
  entries: readonly (ContextMenuItem | ContextMenuGroup)[],
  scope = "ContextMenu",
): ContextMenuItem[] {
  const items: ContextMenuItem[] = [];
  const separator: PredefinedMenuItemOptions = { item: "Separator" };
  for (const entry of entries) {
    if (isMenuGroup(entry)) {
      if (!entry.length) continue;
      if (items.length > 0) items.push(separator);
      for (const item of entry) items.push(processMenuItem(item, scope));
    } else {
      items.push(processMenuItem(entry, scope));
    }
  }
  return items;
}

function toMenuPosition(pos?: PopupMenuPosition): PhysicalPosition | LogicalPosition | Position | undefined {
  if (!pos) return undefined;
  if (pos instanceof LogicalPosition || pos instanceof PhysicalPosition || pos instanceof Position)
    return pos;
  return "x" in pos && "y" in pos ? new LogicalPosition(pos.x, pos.y) : undefined;
}

export async function popupContextMenu(
  items: readonly (ContextMenuItem | ContextMenuGroup)[],
  options: PopupContextMenuOptions = {},
): Promise<void> {
  const scope = options.scope ?? "ContextMenu";
  const processed = flattenMenuGroups(items, scope);
  if (!processed.length) return;

  const popupPos = toMenuPosition(options.position);
  try {
    const menu = await Menu.new({ items: processed });
    try {
      await menu.popup(popupPos, options.window ?? getCurrentWindow());
    } finally {
      await menu.close().catch((err) => logError(`[${scope}] menu close failed: ${String(err)}`));
    }
  } catch (err) {
    void logError(`[${scope}] menu popup failed: ${String(err)}`);
  }
}

export function menuItem(
  text: string,
  action: () => void | Promise<void>,
  options: { enabled?: boolean; accelerator?: string; scope?: string } = {},
): MenuItemOptions {
  const item: MenuItemOptions = { text, enabled: options.enabled ?? true };
  const wrappedAction = wrapAction(options.scope ?? "MenuItem", action);
  if (wrappedAction) item.action = wrappedAction;
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
  const item: CheckMenuItemOptions = { text, checked, enabled: options.enabled ?? true };
  const wrappedAction = wrapAction(options.scope ?? "CheckMenuItem", action);
  if (wrappedAction) item.action = wrappedAction;
  return item;
}

export function submenu(text: string, items: ContextMenuItem[]): SubmenuOptions {
  return { text, items };
}

export function separatorItem(): PredefinedMenuItemOptions {
  return { item: "Separator" };
}
