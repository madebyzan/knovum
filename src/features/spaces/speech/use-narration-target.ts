import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef, watch } from "vue";

import { availableExecutionTargets, defaultExecutionTarget } from "../../../foundation/ai/ai-execution";
import { canStartWith } from "../../../foundation/ai/ai-readiness";
import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { useAiReadiness } from "../../../foundation/ai/use-ai-readiness";
import { useExecutionCatalogQuery } from "../../../foundation/ai/use-execution-catalog-query";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";

/**
 * Resolves available AI execution targets (API models and enabled desktop agents) for narration
 * script preparation, keeping the active target synchronized with Knovum AI readiness checks.
 */
export function useNarrationTarget() {
  const store = useAiSettingsStore();
  const catalog = useExecutionCatalogQuery({ detectInstallations: false });
  const { settings, ready: settingsReady } = storeToRefs(store);
  const selectedValue = shallowRef("");

  onMounted(() => void store.ensureLoaded());

  const targets = computed(() => {
    if (!settingsReady.value) return [];
    const agents = catalog.data.value?.agents ?? [];
    return availableExecutionTargets(settings.value, "writing").map((option) => ({
      value: option.value,
      target: option.target,
      label:
        option.target.type === "agent"
          ? (agents.find(({ kind }) => option.target.type === "agent" && kind === option.target.runtimeKind)
              ?.displayName ?? option.label)
          : option.label,
    }));
  });

  watch(
    [settingsReady, targets],
    () => {
      if (!settingsReady.value) return;
      if (selectedValue.value && targets.value.some(({ value }) => value === selectedValue.value)) return;
      const preferred = defaultExecutionTarget(settings.value, "writing");
      const preferredValue = preferred ? JSON.stringify(preferred) : "";
      const matched = targets.value.find(({ value }) => value === preferredValue) ?? targets.value[0];
      selectedValue.value = matched?.value ?? "";
    },
    { immediate: true },
  );

  const selectedTarget = computed<TaskTarget | undefined>(
    () => targets.value.find(({ value }) => value === selectedValue.value)?.target,
  );
  const options = computed(() => targets.value.map(({ value, label }) => ({ value, label })));
  const readiness = useAiReadiness(() => [
    { kind: "routed", purpose: "writing", target: selectedTarget.value },
  ]);
  const ready = computed(() => canStartWith(readiness.value));

  function selectValue(value: string) {
    selectedValue.value = value;
  }

  return { options, selectedValue, selectedTarget, readiness, ready, selectValue };
}
