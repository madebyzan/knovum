import { uniq } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed } from "vue";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";

export function useProtectedExecutionTargets() {
  const { settings } = storeToRefs(useAiSettingsStore());
  const targets = computed<TaskTarget[]>(() =>
    Object.values(settings.value.defaults).filter((target): target is TaskTarget => Boolean(target)),
  );

  return {
    protectedConnectionIds: computed(() =>
      uniq(targets.value.filter((target) => target.type === "model").map(({ connectionId }) => connectionId)),
    ),
    protectedModelTargets: computed(() =>
      targets.value.filter((target) => target.type === "model").map((target) => JSON.stringify(target)),
    ),
  };
}
