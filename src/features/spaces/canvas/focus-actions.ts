import type { SpacePlacement } from "./navigation-requests";
import type { SpaceSavedViewFilter } from "../knowledge/model/projections";
import { snapshotSpaceQueryFilter } from "../knowledge/model/query-filter";
import {
  findSemanticRelationPath,
  queryMatchingNodes,
  relationNeighborhood,
} from "../knowledge/model/relation-graph";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceState } from "../session/session-state";

export function createCanvasFocusActions(state: SpaceState, placement: SpacePlacement) {
  function focusSection(id: string) {
    state.canvasFocusIds.value = state.nodes.value
      .filter((node) => node.id === id || node.data.sectionId === id)
      .map((node) => node.id);
  }

  function focusNeighborhood(id: string, depth = 1) {
    const ids = relationNeighborhood(state.relations.value, [id], depth);
    includeParentSections(ids, state.nodes.value);
    state.canvasFocusIds.value = [...ids];
  }

  function focusRelationPath(sourceId: string, targetId: string) {
    const path = findSemanticRelationPath(state.relations.value, sourceId, targetId);
    if (!path.length) return false;
    const ids = new Set(path);
    includeParentSections(ids, state.nodes.value);
    state.canvasFocusIds.value = [...ids];
    return true;
  }

  async function openSceneView(id: string) {
    const scene = state.scenes.value.find((candidate) => candidate.id === id);
    if (!scene) return;
    const viewport = { x: scene.viewportX, y: scene.viewportY, zoom: scene.viewportZoom };
    state.relationKindFilter.value = [...scene.relationKinds];
    state.relationStatusFilter.value = [...scene.relationStatuses];
    state.savedQueryFilter.value = snapshotSpaceQueryFilter(scene.queryFilter);
    const dynamicIds = hasQueryFilter(scene.queryFilter)
      ? queryMatchingNodes(state.nodes.value, state.relations.value, scene.queryFilter).map((node) => node.id)
      : scene.focusNodeIds;
    const ids = new Set<string>(dynamicIds as string[]);
    includeParentSections(ids, state.nodes.value);
    state.canvasFocusIds.value = [...ids];
    placement.requestViewport(viewport);
    placement.announceScene(scene, "opened");
  }

  return {
    focusSection,
    focusNeighborhood,
    focusRelationPath,
    clearCanvasFocus: () => {
      state.canvasFocusIds.value = [];
    },
    openSceneView,
  };
}

function includeParentSections(ids: Set<string>, nodes: readonly SpaceCanvasNode[]) {
  for (const node of nodes) {
    if (ids.has(node.id) && node.data.sectionId) ids.add(node.data.sectionId);
  }
}

function hasQueryFilter(filter: SpaceSavedViewFilter) {
  return Object.values(filter).some((value) =>
    Array.isArray(value) ? value.length > 0 : value !== undefined && value !== "",
  );
}
