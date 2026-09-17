import { storeToRefs } from "pinia";
import { computed } from "vue";

import type { AgentKind, AgentTarget } from "../../foundation/ai/agent-models";
import type { AiModelTarget, ExecutionTarget } from "../../foundation/ai/ai-models";
import { useAiSettingsStore } from "../../foundation/ai/ai-settings-store";
import { useCreationLibraryStore } from "../../foundation/ai/creation-library-store";

export function useProtectedExecutionTargets() {
  const aiStore = useAiSettingsStore();
  const creationStore = useCreationLibraryStore();
  const { settings } = storeToRefs(aiStore);
  const { library } = storeToRefs(creationStore);
  const targets = computed<ExecutionTarget[]>(() => [
    ...Object.values(settings.value.defaults).filter((target): target is ExecutionTarget => Boolean(target)),
    ...library.value.generationPresets.map(({ target }) => target),
    ...library.value.creationWorkflows.flatMap((workflow) =>
      Object.values(workflow.targets).map(({ target }) => target),
    ),
  ]);
  const modelTargets = computed(() =>
    targets.value.filter((target): target is AiModelTarget => target.type === "model"),
  );
  const agentTargets = computed(() =>
    targets.value.filter((target): target is AgentTarget => target.type === "agent"),
  );

  return {
    protectedConnectionIds: computed(() => [
      ...new Set(modelTargets.value.map(({ connectionId }) => connectionId)),
    ]),
    protectedModelTargets: computed(() =>
      modelTargets.value.map(({ connectionId, modelId }) => `${connectionId}:${modelId}`),
    ),
    protectedAgentKinds: computed<AgentKind[]>(() => [
      ...new Set(agentTargets.value.map(({ kind }) => kind)),
    ]),
  };
}
