import { openUrl } from "@tauri-apps/plugin-opener";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../foundation/errors/error-format";
import { i18n } from "../foundation/i18n/instance";
import { linkDestination } from "../shared/contracts/link-destination";

/**
 * The window shows the app and nothing else, so following a link never navigates it: an external
 * link opens in the system's handler, and any other destination goes nowhere. A surface that can
 * follow a local link — a conversation — handles the click and prevents its default first; a link
 * inside editable text is the editor's to place the caret in.
 */
function follow(event: MouseEvent): void {
  const link = event.target instanceof Element ? event.target.closest("a[href]") : null;
  if (!(link instanceof HTMLAnchorElement) || event.defaultPrevented || link.isContentEditable) return;
  event.preventDefault();
  const href = link.getAttribute("href") ?? "";
  if (linkDestination(href) !== "external") return;
  openUrl(href).catch((error: unknown) =>
    toast.error(i18n.global.t("common.linkOpenFailed"), { description: getErrorMessage(error) }),
  );
}

export function installLinkPolicy(): () => void {
  // `click` covers modified clicks; a middle click arrives only as `auxclick`.
  document.addEventListener("click", follow);
  document.addEventListener("auxclick", follow);
  return () => {
    document.removeEventListener("click", follow);
    document.removeEventListener("auxclick", follow);
  };
}
