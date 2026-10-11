import { isEditableTarget } from "../foundation/dom/editable-target";

function preventBrowserMenu(event: MouseEvent): void {
  if (!isEditableTarget(event.target)) event.preventDefault();
}

export function installContextMenuPolicy(): (() => void) | undefined {
  if (import.meta.env.DEV) return;

  document.addEventListener("contextmenu", preventBrowserMenu, { capture: true });
  return () => document.removeEventListener("contextmenu", preventBrowserMenu, { capture: true });
}
