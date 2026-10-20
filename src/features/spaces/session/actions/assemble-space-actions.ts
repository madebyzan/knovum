import type { createSpaceMediaActions } from "../../media/actions";
import {
  createSpaceScene,
  duplicateSpaceScene,
  reorderSpaceScenes,
  removeSpaceScene,
  updateSpaceScene,
} from "../../scenes/scene-actions";
import { searchSpace } from "../../search/search-actions";
import { archiveSpace, permanentlyDeleteSpace, unarchiveSpace } from "../../workspace/archive-actions";
import { showSpaceLibrary } from "../../workspace/library-actions";
import type { createRightSidebarActions } from "../../workspace/right-sidebar-actions";
import { favoriteSpace, unfavoriteSpace } from "../../workspace/space-operations";
import { refreshSpace } from "../../workspace/space-session-actions";
import type { SpaceSessionContext } from "../../workspace/space-session-context";
import { renameSpace } from "../../workspace/space-title-actions";
import { commitActiveSpaceSession, transitionActiveSpaceSession } from "../../workspace/space-transition";
import {
  closeOtherWorkbenchTabs,
  closeWorkbenchTab,
  closeWorkbenchTabsToRight,
} from "../../workspace/workbench-close-actions";
import {
  activateWorkbenchTab,
  createSpace,
  initializeSpaces,
  navigateWorkbenchHistory,
  openSpace as navigateToSpace,
  openSpaceCanvas,
  reopenClosedWorkbenchTab,
} from "../../workspace/workbench-navigation-actions";
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
  const openSpace = async (spaceId: string) => {
    await navigateToSpace(spaceContext, spaceId);
  };
  const refreshActiveSpace = async () => {
    await refreshSpace(spaceContext);
  };
  return {
    initialize,
    commitActiveSession: () => commitActiveSpaceSession(spaceContext),
    commitWorkspaceNavigation: () => transitionActiveSpaceSession(spaceContext, async () => {}),
    createSpace: (title: string) => createSpace(spaceContext, title),
    showSpaceLibrary: (scope: Parameters<typeof showSpaceLibrary>[2]) =>
      showSpaceLibrary(state, spaceContext, scope),
    focusSpaceLibrarySearch: () => (state.librarySearchNonce.value += 1),
    toggleSpaceFavorite: async (spaceId: string) => {
      const space = state.spaces.value.find((candidate) => candidate.id === spaceId);
      if (!space) return;
      const favorite = !space.favorite;
      if (favorite) await favoriteSpace(spaceId);
      else await unfavoriteSpace(spaceId);
      Object.assign(space, { favorite, updatedAt: Date.now() });
    },
    openSpace,
    openSpaceCanvas: async () => {
      await openSpaceCanvas(spaceContext);
    },
    activateTab: async (tabId: string) => {
      await activateWorkbenchTab(spaceContext, tabId);
    },
    closeTab: (tabId: string) => closeWorkbenchTab(spaceContext, tabId),
    closeOtherTabs: (tabId: string) => closeOtherWorkbenchTabs(spaceContext, tabId),
    closeTabsToRight: (tabId: string) => closeWorkbenchTabsToRight(spaceContext, tabId),
    reopenClosedTab: async () => {
      await reopenClosedWorkbenchTab(spaceContext);
    },
    navigateWorkbenchHistory: (direction: -1 | 1) => navigateWorkbenchHistory(spaceContext, direction),
    refreshActiveSpace,
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
