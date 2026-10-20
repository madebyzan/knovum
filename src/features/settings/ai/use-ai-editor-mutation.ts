import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";

/** Owns the admitted write that editor-close and settings navigation must wait for. */
export function useAiEditorMutation() {
  const { t } = useI18n();
  const busy = shallowRef(false);
  const error = shallowRef("");
  let pending: Promise<boolean> | undefined;

  function run(mutation: () => Promise<boolean>) {
    if (pending) return pending;
    busy.value = true;
    error.value = "";
    const admitted = Promise.resolve()
      .then(mutation)
      .catch((cause) => {
        error.value = `${t("settings.general.saveFailed")}: ${getErrorMessage(cause)}`;
        return false;
      })
      .finally(() => {
        busy.value = false;
        if (pending === admitted) pending = undefined;
      });
    pending = admitted;
    return admitted;
  }

  return { busy, error, run, pending: () => pending };
}
