import { commitActiveCanvasThumbnail } from "../../canvas/thumbnail-scheduler";
import { refreshEnrichmentTasks } from "../../enrichment/job-observation";
import type { createSpaceMediaActions } from "../../media/actions";
import type { createRightSidebarActions } from "../../right-sidebar/right-sidebar-actions";
import {
  createSpaceScene,
  duplicateSpaceScene,
  moveSpaceScene,
  reorderSpaceScenes,
  removeSpaceScene,
  updateSpaceScene,
} from "../../scenes/scene-actions";
import { searchSpace } from "../../search/search-actions";
import { archiveSpace, permanentlyDeleteSpace, unarchiveSpace } from "../../workspace/archive-actions";
import { showSpaceLibrary } from "../../workspace/library-actions";
import { setSpaceFavorite } from "../../workspace/space-operations";
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
} from "../../workspace/space-session-actions";
import type { SpaceSessionContext } from "../../workspace/space-session-context";
import { renameSpace } from "../../workspace/space-title-actions";
import { commitActiveSpaceSession } from "../../workspace/space-transition";
import type { SpaceState } from "../session-state";
import type { createSpaceMutations } from "./node-mutations";
import type { createPlacementActions } from "./placement-actions";
import {
  changeSpaceRelationKind,
  createSpaceRelation,
  removeSpaceRelations,
  removeSpaceRelation,
  updateSpaceRelation,
  type RelationContext,
} from "./relation-actions";
import type { createStructureTransferActions } from "./structure-transfer-actions";
import type { createSpaceViewActions } from "./view-actions";

interface Input {
  state: SpaceState;
  mutations: ReturnType<typeof createSpaceMutations>;
  relation: RelationContext;
  placementActions: ReturnType<typeof createPlacementActions>;
  media: ReturnType<typeof createSpaceMediaActions>;
  structureTransfer: ReturnType<typeof createStructureTransferActions>;
  context: ReturnType<typeof createRightSidebarActions>;
  spaceContext: SpaceSessionContext;
  view: ReturnType<typeof createSpaceViewActions>;
}

export function assembleSpaceActions(input: Input) {
  const {
    state,
    mutations,
    relation,
    placementActions,
    media,
    structureTransfer,
    context,
    spaceContext,
    view,
  } = input;
  const navigation = {
    activeSpaceId: state.activeSpaceId,
    scenes: state.scenes,
    editSnapshot: state.editSnapshot,
    focusNodeIds: state.canvasFocusIds,
    relationKinds: state.relationKindFilter,
    relationStatuses: state.relationStatusFilter,
    queryFilter: state.savedQueryFilter,
  };
  const refreshTasks = () => refreshEnrichmentTasks(state.activeSpaceId.value);
  const initialize = async (title: string) => {
    await initializeSpaces(spaceContext, title);
    await refreshTasks();
  };
  const openSpace = async (id: string) => {
    await commitActiveCanvasThumbnail(state);
    state.workspaceMode.value = "space";
    await navigateToSpace(spaceContext, id);
    await refreshTasks();
  };
  const refreshActiveSpace = async () => {
    await refreshSpace(spaceContext);
    await refreshTasks();
  };
  const beginUrlCapture = (capture: Omit<(typeof state.urlCaptures.value)[number], "id">) => {
    const projection = { ...capture, id: crypto.randomUUID() };
    state.urlCaptures.value = [...state.urlCaptures.value, projection];
    return projection;
  };
  const finishUrlCapture = (id: string) => {
    state.urlCaptures.value = state.urlCaptures.value.filter((capture) => capture.id !== id);
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
    state.urlCaptures.value = state.urlCaptures.value.map((capture) =>
      capture.id === id ? { ...capture, ...patch } : capture,
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
    search: (value: string, semantic = false, signal?: AbortSignal) =>
      searchSpace({ context: navigation, value, semantic, signal }),
    createScene: (title: string, viewport: { x: number; y: number; zoom: number }, thumbnailPath: string) =>
      createSpaceScene({ context: navigation, title, viewport, thumbnailPath }),
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
    revokeRelation: (id: string) => removeSpaceRelation(relation, id),
    revokeRelations: (ids: readonly string[]) => removeSpaceRelations(relation, ids),
    updateRelationKind: (id: string, kind: Parameters<typeof changeSpaceRelationKind>[2]) =>
      changeSpaceRelationKind(relation, id, kind),
    updateRelation: (id: string, patch: Parameters<typeof updateSpaceRelation>[2]) =>
      updateSpaceRelation(relation, id, patch),
    ...mutations,
    ...placementActions,
    ...media,
    ...structureTransfer,
    ...context,
    ...view,
  };
}
