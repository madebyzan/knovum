import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { useSpaceStore } from "../session/space-store";

export function useSpaceLibraryCommands() {
  const store = useSpaceStore();
  const { t } = useI18n();

  return computed<readonly AppCommand[]>(() => [
    {
      id: "spaces.find",
      title: t("spaces.space.search.placeholder"),
      shortcut: "$mod+F",
      category: "navigation",
      run: () => {
        store.focusSpaceLibrarySearch();
      },
    },
  ]);
}
