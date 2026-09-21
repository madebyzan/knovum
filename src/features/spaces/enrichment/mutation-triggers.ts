import { watch } from "vue";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import type { createPlacementActions } from "../canvas/placement-actions";
import type { createSpaceMutations } from "../nodes/node-mutations";
import type { SpaceState } from "../session/session-state";
import { resumePendingEnrichmentJobs } from "./job-scheduler";

type SpaceMutations = ReturnType<typeof createSpaceMutations>;
type PlacementActions = ReturnType<typeof createPlacementActions>;

export function withEnrichmentTriggers(
  state: SpaceState,
  mutations: SpaceMutations,
  placementActions: PlacementActions,
) {
  const enrichmentContext = { applyNodeDataForSpace: mutations.applyNodeDataForSpace };
  const ai = useAiSettingsStore();
  watch(
    () => [ai.settings.defaults.visualUnderstanding, ai.settings.connections, ai.settings.desktopAgents],
    () => {
      void resumePendingEnrichmentJobs(enrichmentContext, state.activeSpaceId.value);
    },
    { deep: true },
  );

  return {
    context: enrichmentContext,
    addItem: placementActions.addItem,
    addItems: placementActions.addItems,
    saveNodeData: mutations.saveNodeData,
    saveNodeDataForSpace: mutations.saveNodeDataForSpace,
    applyNodeDataForSpace: mutations.applyNodeDataForSpace,
  };
}
