import {
  checkMenuItem,
  menuItem,
  popupContextMenu,
  type ContextMenuItem,
  type PopupContextMenuOptions,
} from "../../../foundation/context-menu/popup";

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

export function popupSpaceContextMenu(
  items: readonly (ContextMenuItem | readonly ContextMenuItem[])[],
  options: Omit<PopupContextMenuOptions, "scope"> = {},
) {
  return popupContextMenu(items, { ...options, scope: "Space" });
}
