import type { createCanvasFocusActions } from "../canvas/focus-actions";
import type { createPlacementActions } from "../canvas/placement-actions";
import { commitActiveCanvasThumbnail } from "../canvas/thumbnail-scheduler";
import { resumePendingEnrichmentJobs } from "../enrichment/job-scheduler";
import { withEnrichmentTriggers } from "../enrichment/mutation-triggers";
import type { createInspectorActions } from "../inspector/inspector-actions";
import {
  changeSpaceRelationKind,
  createSpaceRelation,
  removeSpaceRelations,
  removeSpaceRelation,
  updateSpaceRelation,
  type RelationContext,
} from "../knowledge/relations/relation-actions";
import type { createSpaceMediaActions } from "../media/actions";
import type { createSpaceNodeLifecycleActions } from "../nodes/node-lifecycle-actions";
import type { createSpaceMutations } from "../nodes/node-mutations";
import type { createSelectionActions } from "../organization/selection-actions";
import {
  createSpaceScene,
  duplicateSpaceScene,
  moveSpaceScene,
  reorderSpaceScenes,
  removeSpaceScene,
  updateSpaceScene,
} from "../scenes/scene-actions";
import { searchSpace } from "../search/search-actions";
import { archiveSpace, permanentlyDeleteSpace, unarchiveSpace } from "../workspace/archive-actions";
import { showSpaceLibrary } from "../workspace/library-actions";
import { setSpaceFavorite } from "../workspace/space-repository";
import {
  closeOtherSpaceTabs,
  closeSpaceTab,
  closeSpaceTabsToRight,
  createSpace,
  initializeSpaces,
  navigateSpaceHistory,
  openSpace as navigateToSpace,
  refreshSpace,
  reopenClosedSpaceTab,
  renameSpace,
  type SpaceSessionContext,
} from "../workspace/space-session-actions";
import { commitActiveSpaceSession } from "../workspace/space-transition";
import type { SpaceState } from "./session-state";

interface Input {
  state: SpaceState;
  mutations: ReturnType<typeof createSpaceMutations>;
  relation: RelationContext;
  placementActions: ReturnType<typeof createPlacementActions>;
  organization: ReturnType<typeof createSelectionActions>;
  media: ReturnType<typeof createSpaceMediaActions>;
  lifecycle: ReturnType<typeof createSpaceNodeLifecycleActions>;
  context: ReturnType<typeof createInspectorActions>;
  spaceContext: SpaceSessionContext;
  view: ReturnType<typeof createCanvasFocusActions>;
}

export function assembleSpaceActions(input: Input) {
  const {
    state,
    mutations,
    relation,
    placementActions,
    organization,
    media,
    lifecycle,
    context,
    spaceContext,
    view,
  } = input;
  const navigation = {
    activeSpaceId: state.activeSpaceId,
    scenes: state.scenes,
    focusNodeIds: state.canvasFocusIds,
    relationKinds: state.relationKindFilter,
    relationStatuses: state.relationStatusFilter,
    queryFilter: state.savedQueryFilter,
  };
  const { context: enrichmentContext, ...enrichmentActions } = withEnrichmentTriggers(
    state,
    mutations,
    placementActions,
  );
  const resumeEnrichment = () => resumePendingEnrichmentJobs(enrichmentContext, state.activeSpaceId.value);
  const initialize = async (title: string) => {
    await initializeSpaces(spaceContext, title);
    await resumeEnrichment();
  };
  const openSpace = async (id: string) => {
    await commitActiveCanvasThumbnail(state);
    state.workspaceMode.value = "space";
    await navigateToSpace(spaceContext, id);
    await resumeEnrichment();
  };
  const refreshActiveSpace = async () => {
    await refreshSpace(spaceContext);
    await resumeEnrichment();
  };
  const beginUrlCapture = (capture: Omit<(typeof state.urlCaptures.value)[number], "id">) => {
    const projection = { ...capture, id: crypto.randomUUID() };
    state.urlCaptures.value = [...state.urlCaptures.value, projection];
    return projection;
  };
  const finishUrlCapture = (id: string) => {
    state.urlCaptures.value = state.urlCaptures.value.filter((item) => item.id !== id);
  };
  const updateUrlCapture = (
    id: string,
    patch: Partial<
      Pick<
        (typeof state.urlCaptures.value)[number],
        "status" | "downloadedBytes" | "totalBytes" | "processedMs" | "durationMs"
      >
    >,
  ) => {
    state.urlCaptures.value = state.urlCaptures.value.map((item) =>
      item.id === id ? { ...item, ...patch } : item,
    );
  };
  return {
    initialize,
    commitActiveSession: () => commitActiveSpaceSession(spaceContext),
    createSpace: async (title: string) => {
      await commitActiveCanvasThumbnail(state);
      await createSpace(spaceContext, title);
      state.workspaceMode.value = "space";
    },
    showSpaceLibrary: (scope: Parameters<typeof showSpaceLibrary>[1]) => showSpaceLibrary(state, scope),
    focusSpaceLibrarySearch: () => (state.librarySearchNonce.value += 1),
    setSpaceFavorite: async (id: string, favorite: boolean) => {
      await setSpaceFavorite(id, favorite);
      const space = state.spaces.value.find((candidate) => candidate.id === id);
      if (space) Object.assign(space, { favorite, updatedAt: Date.now() });
    },
    openSpace,
    closeSpaceTab: (id: string) => closeSpaceTab(spaceContext, id),
    closeOtherSpaceTabs: (id: string) => closeOtherSpaceTabs(spaceContext, id),
    closeSpaceTabsToRight: (id: string) => closeSpaceTabsToRight(spaceContext, id),
    reopenClosedSpaceTab: () => reopenClosedSpaceTab(spaceContext),
    navigateSpaceHistory: (direction: -1 | 1) => navigateSpaceHistory(spaceContext, direction),
    refreshActiveSpace,
    beginUrlCapture,
    updateUrlCapture,
    finishUrlCapture,
    renameSpace: (id: string, title: string) => renameSpace(spaceContext, id, title),
    archiveSpace: (id: string) => archiveSpace(spaceContext, id),
    unarchiveSpace: (id: string) => unarchiveSpace(spaceContext, id),
    permanentlyDeleteSpace: (id: string) => permanentlyDeleteSpace(spaceContext, id),
    search: (value: string) => searchSpace(navigation, value),
    createScene: (title: string, viewport: { x: number; y: number; zoom: number }, thumbnailPath: string) =>
      createSpaceScene(navigation, title, viewport, thumbnailPath),
    updateScene: (id: string, patch: Parameters<typeof updateSpaceScene>[2]) =>
      updateSpaceScene(navigation, id, patch),
    duplicateScene: (id: string) => duplicateSpaceScene(navigation, id),
    moveScene: (id: string, direction: -1 | 1) => moveSpaceScene(navigation, id, direction),
    reorderScenes: (sourceId: string, targetId: string) => reorderSpaceScenes(navigation, sourceId, targetId),
    removeScene: (id: string) => removeSpaceScene(navigation, id),
    createRelation: (
      sourceId: string,
      targetId: string,
      kind: Parameters<typeof createSpaceRelation>[1]["kind"],
      options: { rationale?: string; spaceId?: string } = {},
    ) => createSpaceRelation(relation, { sourceId, targetId, kind, ...options }),
    removeRelation: (id: string) => removeSpaceRelation(relation, id),
    removeRelations: (ids: readonly string[]) => removeSpaceRelations(relation, ids),
    updateRelationKind: (id: string, kind: Parameters<typeof changeSpaceRelationKind>[2]) =>
      changeSpaceRelationKind(relation, id, kind),
    updateRelation: (id: string, patch: Parameters<typeof updateSpaceRelation>[2]) =>
      updateSpaceRelation(relation, id, patch),
    ...mutations,
    ...placementActions,
    ...organization,
    ...media,
    ...lifecycle,
    ...context,
    ...view,
    ...enrichmentActions,
  };
}
