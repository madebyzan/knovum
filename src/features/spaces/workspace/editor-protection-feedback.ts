import { toast } from "vue-sonner";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { useSpaceStore } from "../session/space-store";
import { EditorResourceProtectedError } from "./editor-resource-protection";

export function reportEditorProtection(error: unknown) {
  const tab = error instanceof EditorResourceProtectedError ? error.tab : undefined;
  if (!tab && decodeAppError(error)?.code !== "space_editor_resource_protected") return false;
  toast.info(
    i18n.global.t("spaces.editor.tabs.protected"),
    tab
      ? {
          action: {
            label: i18n.global.t("spaces.editor.tabs.goToEditor"),
            onClick: () => void revealEditor(tab.id),
          },
        }
      : {},
  );
  return true;
}

export async function runEditorMutation(operation: () => void | Promise<void>) {
  try {
    await operation();
  } catch (error) {
    if (!reportEditorProtection(error)) throw error;
  }
}

async function revealEditor(tabId: string) {
  const store = useSpaceStore();
  try {
    await store.activateTab(tabId);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
