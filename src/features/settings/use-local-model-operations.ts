import { computed, shallowRef, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { decodeAppError } from "../../foundation/errors/app-error";
import { getErrorMessage } from "../../foundation/errors/error-format";
import { removeLocalModel, selectLocalModel } from "../../foundation/runtime/local-resources";
import { startResourcePreparation } from "../../foundation/runtime/resource-preparation";
import { invalidateLocalResources } from "../../foundation/runtime/use-resource-queries";
import type { LocalModelAvailability } from "../../shared/contracts/generated/LocalModelAvailability";
import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import { confirmAction } from "../../shared/ui/confirm-action";

export function useLocalModelOperations(model: Readonly<Ref<LocalModelAvailability>>) {
  const { t } = useI18n();
  const preparation = shallowRef<ReturnType<typeof startResourcePreparation>>();
  const progress = shallowRef<ResourcePreparationProgress>();
  const operation = shallowRef<"select" | "remove">();
  const cancelling = shallowRef(false);
  const error = shallowRef<string>();
  const busy = computed(() => Boolean(operation.value || preparation.value));

  async function select() {
    if (busy.value) return;
    operation.value = "select";
    error.value = undefined;
    try {
      await selectLocalModel(model.value.id);
      await invalidateLocalResources();
    } catch (cause) {
      error.value = getErrorMessage(cause);
    } finally {
      operation.value = undefined;
    }
  }

  async function download() {
    if (busy.value) return;
    error.value = undefined;
    progress.value = undefined;
    try {
      const handle = startResourcePreparation(
        { kind: "model", model: model.value.id },
        t("settings.ai.localModels.downloading", { name: model.value.name }),
        (update) => {
          progress.value = update;
        },
      );
      preparation.value = handle;
      await handle.completion;
    } catch (cause) {
      if (decodeAppError(cause)?.code !== "local_resource_preparation_cancelled")
        error.value = getErrorMessage(cause);
    } finally {
      preparation.value = undefined;
      cancelling.value = false;
    }
  }

  async function cancel() {
    if (!preparation.value || cancelling.value) return;
    cancelling.value = true;
    error.value = undefined;
    try {
      await preparation.value.cancel();
    } catch (cause) {
      error.value = getErrorMessage(cause);
      cancelling.value = false;
    }
  }

  async function remove() {
    if (busy.value) return;
    operation.value = "remove";
    error.value = undefined;
    try {
      if (
        !(await confirmAction({
          title: t("settings.ai.localModels.removeTitle", { name: model.value.name }),
          description: t("settings.ai.localModels.removeDescription"),
          confirmLabel: t("common.remove"),
        }))
      )
        return;
      await removeLocalModel(model.value.id);
      await invalidateLocalResources();
    } catch (cause) {
      error.value = getErrorMessage(cause);
    } finally {
      operation.value = undefined;
    }
  }

  return { preparation, progress, operation, cancelling, error, busy, select, download, cancel, remove };
}
