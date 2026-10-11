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
import { commitActiveSpaceSession, transitionActiveSpaceSession } from "../../workspace/space-transition";
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

interface SpaceActionsInput {
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

export function assembleSpaceActions(input: SpaceActionsInput) {
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
  const initialize = (title: string) => initializeSpaces(spaceContext, title);
  const openSpace = (spaceId: string) => navigateToSpace(spaceContext, spaceId);
  const refreshActiveSpace = async () => {
    await refreshSpace(spaceContext);
  };
  const beginUrlCapture = (capture: Omit<(typeof state.urlCaptures.value)[number], "id">) => {
    const projection = { ...capture, id: crypto.randomUUID() };
    state.urlCaptures.value = [...state.urlCaptures.value, projection];
    return projection;
  };
  const finishUrlCapture = (captureId: string) => {
    state.urlCaptures.value = state.urlCaptures.value.filter((capture) => capture.id !== captureId);
  };
  const updateUrlCapture = (
    captureId: string,
    patch: Partial<
      Pick<
        (typeof state.urlCaptures.value)[number],
        "status" | "downloadedBytes" | "totalBytes" | "processedMs" | "durationMs"
      >
    >,
  ) => {
    state.urlCaptures.value = state.urlCaptures.value.map((capture) =>
      capture.id === captureId ? { ...capture, ...patch } : capture,
    );
  };
  return {
    initialize,
    commitActiveSession: () => commitActiveSpaceSession(spaceContext),
    commitWorkspaceNavigation: () => transitionActiveSpaceSession(spaceContext, async () => {}),
    createSpace: (title: string) => createSpace(spaceContext, title),
    showSpaceLibrary: (scope: Parameters<typeof showSpaceLibrary>[2]) =>
      showSpaceLibrary(state, spaceContext, scope),
    focusSpaceLibrarySearch: () => (state.librarySearchNonce.value += 1),
    setSpaceFavorite: async (spaceId: string, favorite: boolean) => {
      await setSpaceFavorite(spaceId, favorite);
      const space = state.spaces.value.find((candidate) => candidate.id === spaceId);
      if (space) Object.assign(space, { favorite, updatedAt: Date.now() });
    },
    openSpace,
    closeSpaceTab: (spaceId: string) => closeSpaceTab(spaceContext, spaceId),
    closeOtherSpaceTabs: (spaceId: string) => closeOtherSpaceTabs(spaceContext, spaceId),
    closeSpaceTabsToRight: (spaceId: string) => closeSpaceTabsToRight(spaceContext, spaceId),
    reopenClosedSpaceTab: () => reopenClosedSpaceTab(spaceContext),
    navigateSpaceHistory: (direction: -1 | 1) => navigateSpaceHistory(spaceContext, direction),
    refreshActiveSpace,
    beginUrlCapture,
    updateUrlCapture,
    finishUrlCapture,
    renameSpace: (spaceId: string, title: string) => renameSpace(spaceContext, spaceId, title),
    archiveSpace: (spaceId: string) => archiveSpace(spaceContext, spaceId),
    unarchiveSpace: (spaceId: string) => unarchiveSpace(spaceContext, spaceId),
    permanentlyDeleteSpace: (spaceId: string) => permanentlyDeleteSpace(spaceContext, spaceId),
    search: (value: string, semantic = false, signal?: AbortSignal) =>
      searchSpace({ context: navigation, value, semantic, signal }),
    createScene: (title: string, viewport: { x: number; y: number; zoom: number }, thumbnailPath: string) =>
      createSpaceScene({ context: navigation, title, viewport, thumbnailPath }),
    updateScene: (sceneId: string, patch: Parameters<typeof updateSpaceScene>[2]) =>
      updateSpaceScene(navigation, sceneId, patch),
    duplicateScene: (sceneId: string) => duplicateSpaceScene(navigation, sceneId),
    moveScene: (sceneId: string, direction: -1 | 1) => moveSpaceScene(navigation, sceneId, direction),
    reorderScenes: (sourceId: string, targetId: string) => reorderSpaceScenes(navigation, sourceId, targetId),
    removeScene: (sceneId: string) => removeSpaceScene(navigation, sceneId),
    createRelation: (
      sourceId: string,
      targetId: string,
      kind: Parameters<typeof createSpaceRelation>[1]["kind"],
      options: { rationale?: string; spaceId?: string } = {},
    ) => createSpaceRelation(relation, { sourceId, targetId, kind, ...options }),
    revokeRelation: (relationId: string) => removeSpaceRelation(relation, relationId),
    revokeRelations: (relationIds: readonly string[]) => removeSpaceRelations(relation, relationIds),
    updateRelationKind: (relationId: string, kind: Parameters<typeof changeSpaceRelationKind>[2]) =>
      changeSpaceRelationKind(relation, relationId, kind),
    updateRelation: (relationId: string, patch: Parameters<typeof updateSpaceRelation>[2]) =>
      updateSpaceRelation(relation, relationId, patch),
    ...mutations,
    ...placementActions,
    ...media,
    ...structureTransfer,
    ...context,
    ...view,
  };
}
