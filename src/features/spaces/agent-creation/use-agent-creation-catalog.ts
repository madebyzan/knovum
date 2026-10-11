import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../foundation/ai/use-execution-catalog-query";

export function useAgentCreationCatalog() {
  const { t } = useI18n();
  const { settings } = storeToRefs(useAiSettingsStore());
  const query = useExecutionCatalogQuery();
  const agents = computed(() => {
    const enabled = new Set(settings.value.desktopAgents.map(({ kind }) => kind));
    return (query.data.value?.agents ?? []).filter(({ kind }) => enabled.has(kind));
  });
  const agentOptions = computed(() =>
    agents.value.map(({ kind, displayName }) => {
      const runtime = query.data.value?.installations.find((item) => item.kind === kind);
      return {
        value: kind,
        label: displayName,
        disabled: false,
        description:
          !runtime || runtime.detected ? t("ai.target.desktopAgent") : t("ai.desktopAgent.notDetected"),
      };
    }),
  );
  return { ...query, agents, agentOptions };
}
