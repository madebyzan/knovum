import { watch } from "vue";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import type { createSpaceMutations } from "../session/actions/node-mutations";
import type { createPlacementActions } from "../session/actions/placement-actions";
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
  const resume = (spaceId: string | undefined) => resumePendingEnrichmentJobs(enrichmentContext, spaceId);
  const ai = useAiSettingsStore();
  watch(
    () => [ai.settings.defaults.visualUnderstanding, ai.settings.connections, ai.settings.desktopAgents],
    () => {
      void resume(state.activeSpaceId.value);
    },
    { deep: true },
  );

  const addItem = async (...args: Parameters<typeof placementActions.addItem>) => {
    const spaceId = state.activeSpaceId.value;
    const added = await placementActions.addItem(...args);
    await resume(spaceId);
    return added;
  };

  const addItems = async (...args: Parameters<typeof placementActions.addItems>) => {
    const spaceId = state.activeSpaceId.value;
    const added = await placementActions.addItems(...args);
    await resume(spaceId);
    return added;
  };

  const saveNodeData = async (...args: Parameters<typeof mutations.saveNodeData>) => {
    const spaceId = state.activeSpaceId.value;
    await mutations.saveNodeData(...args);
    await resume(spaceId);
  };

  const saveNodeDataForSpace = async (...args: Parameters<typeof mutations.saveNodeDataForSpace>) => {
    await mutations.saveNodeDataForSpace(...args);
    await resume(args[0]);
  };

  const applyNodeDataForSpace = (...args: Parameters<typeof mutations.applyNodeDataForSpace>) => {
    mutations.applyNodeDataForSpace(...args);
  };

  return {
    context: enrichmentContext,
    addItem,
    addItems,
    saveNodeData,
    saveNodeDataForSpace,
    applyNodeDataForSpace,
  };
}
