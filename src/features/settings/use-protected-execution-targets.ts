import { uniq } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed } from "vue";

import { useAiSettingsStore } from "../../foundation/ai/ai-settings-store";
import { useCreationLibraryStore } from "../../foundation/ai/creation-library-store";
import { encodeExecutionTarget } from "../../foundation/ai/execution-target-codec";
import type { TaskTargetRef } from "../../shared/contracts/generated/TaskTargetRef";

export function useProtectedExecutionTargets() {
  const aiStore = useAiSettingsStore();
  const creationStore = useCreationLibraryStore();
  const { settings } = storeToRefs(aiStore);
  const { library } = storeToRefs(creationStore);
  const targets = computed<TaskTargetRef[]>(() => [
    ...Object.values(settings.value.defaults).filter((target): target is TaskTargetRef => Boolean(target)),
    ...library.value.generationPresets.map(({ target }) => target),
  ]);

  return {
    protectedConnectionIds: computed(() =>
      uniq(targets.value.filter((target) => target.type === "model").map(({ connectionId }) => connectionId)),
    ),
    protectedModelTargets: computed(() =>
      targets.value.filter((target) => target.type === "model").map(encodeExecutionTarget),
    ),
  };
}
