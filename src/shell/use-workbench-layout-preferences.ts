import { error as logError } from "@tauri-apps/plugin-log";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic } from "../foundation/errors/error-format";
import { usePreferencesStore } from "../foundation/preferences/preferences-store";

export function useWorkbenchLayoutPreferences() {
  const { t } = useI18n();
  const preferences = usePreferencesStore();
  const sidebarPinned = shallowRef(preferences.workbench.sidebarVisible);
  const sidebarWidth = shallowRef(preferences.workbench.sidebarWidth);
  const rightSidebarOpen = shallowRef(preferences.workbench.rightSidebarVisible);
  const rightSidebarWidth = shallowRef(preferences.workbench.rightSidebarWidth);

  function persist() {
    void preferences
      .setWorkbenchLayout({
        sidebarVisible: sidebarPinned.value,
        sidebarWidth: sidebarWidth.value,
        rightSidebarVisible: rightSidebarOpen.value,
        rightSidebarWidth: rightSidebarWidth.value,
      })
      .catch((error) => {
        toast.error(t("shell.layoutSaveFailed"), { id: "workbench-layout-save-failed" });
        return logError(`Failed to save workbench layout: ${formatErrorDiagnostic(error)}`);
      });
  }

  return { sidebarPinned, sidebarWidth, rightSidebarOpen, rightSidebarWidth, persist };
}
