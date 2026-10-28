import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, watchEffect, type MaybeRefOrGetter } from "vue";

import { useLocalModelsQuery } from "../runtime/use-local-models-query";
import {
  aiSourcesOf,
  desktopAgentPresence,
  resolveAiReadiness,
  type AiReadiness,
  type AiRequirement,
} from "./ai-readiness";
import { useAiSettingsStore } from "./ai-settings-store";
import { agentDescriptorsQueryOptions, agentInstallationsQueryOptions } from "./use-execution-catalog-query";

/** The live readiness of a feature's requirements, for explaining it beside the control that runs it. */
export function useAiReadiness(requirements: MaybeRefOrGetter<readonly AiRequirement[]>) {
  const store = useAiSettingsStore();
  const settings = computed(() => (store.ready && !store.loadError ? store.settings : undefined));
  const reads = computed(() => aiSourcesOf(toValue(requirements), settings.value));
  const models = useLocalModelsQuery(() => reads.value.has("models"));
  const descriptors = useQuery(
    computed(() => ({ ...agentDescriptorsQueryOptions, enabled: reads.value.has("agents") })),
  );
  const installations = useQuery(
    computed(() => ({ ...agentInstallationsQueryOptions, enabled: reads.value.has("agents") })),
  );
  watchEffect(() => {
    if (reads.value.has("settings")) void store.ensureLoaded();
  });
  const agents = computed(() =>
    descriptors.data.value && installations.data.value
      ? desktopAgentPresence(descriptors.data.value, installations.data.value)
      : undefined,
  );
  return computed<AiReadiness>(() =>
    resolveAiReadiness(toValue(requirements), {
      settings: settings.value,
      settingsFailed: Boolean(store.loadError),
      models: models.data.value,
      modelsFailed: models.isError.value,
      agents: agents.value,
      agentsFailed: descriptors.isError.value || installations.isError.value,
    }),
  );
}
