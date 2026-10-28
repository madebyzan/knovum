import { prefersSystemContextMenu } from "../foundation/dom/text-selection";

function preventBrowserMenu(event: MouseEvent): void {
  if (!prefersSystemContextMenu(event.target)) event.preventDefault();
}

export function installContextMenuPolicy(): (() => void) | undefined {
  if (import.meta.env.DEV) return;

  document.addEventListener("contextmenu", preventBrowserMenu, { capture: true });
  return () => document.removeEventListener("contextmenu", preventBrowserMenu, { capture: true });
}
