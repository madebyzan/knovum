import type { PredefinedMenuItemOptions } from "@tauri-apps/api/menu";

import {
  checkMenuItem,
  menuItem,
  popupContextMenu,
  submenu,
  type ContextMenuItem,
  type PopupContextMenuOptions,
} from "../../../foundation/context-menu/context-menu";

export function spaceMenuItem(
  text: string,
  action: () => void | Promise<void>,
  options: { enabled?: boolean; accelerator?: string } = {},
) {
  return menuItem(text, action, { ...options, scope: "Space" });
}

export function spaceCheckItem(text: string, checked: boolean, action: () => void | Promise<void>) {
  return checkMenuItem(text, checked, action, { scope: "Space" });
}

export function spaceSubmenu(text: string, items: ContextMenuItem[]) {
  return submenu(text, items);
}

export function spacePredefinedItem(
  item: PredefinedMenuItemOptions["item"],
  text?: string,
): PredefinedMenuItemOptions {
  return { item, ...(text !== undefined ? { text } : {}) };
}

export function popupSpaceContextMenu(
  items: readonly (ContextMenuItem | readonly ContextMenuItem[])[],
  options: Omit<PopupContextMenuOptions, "scope"> = {},
) {
  return popupContextMenu(items, { ...options, scope: "Space" });
}
