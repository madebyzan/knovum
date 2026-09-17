import { ask } from "@tauri-apps/plugin-dialog";

import { i18n } from "../../../../foundation/i18n/i18n";

export async function confirmDeleteSpaceSectionContents(remove: () => void | Promise<void>) {
  const t = i18n.global.t;
  const confirmed = await ask(t("spaces.canvas.deleteSectionContentsDescription"), {
    title: t("spaces.canvas.deleteSectionWithContents"),
    kind: "warning",
    okLabel: t("common.delete"),
    cancelLabel: t("common.cancel"),
  });
  if (confirmed) await remove();
}
