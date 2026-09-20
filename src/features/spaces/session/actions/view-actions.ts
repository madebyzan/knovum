import { currentCanvasEditor } from "../../canvas/editor-session";
import type { SpacePlacement } from "../../canvas/navigation-requests";
import type { SpaceSavedViewFilter } from "../../knowledge/model/projections";
import { snapshotSpaceQueryFilter } from "../../knowledge/model/query-filter";
import {
  findSemanticRelationPath,
  queryMatchingNodes,
  relationNeighborhood,
} from "../../knowledge/model/relation-graph";
import { indexStructure, structureAncestors, structureSubtree } from "../../model/structure/structure-index";
import type { SpaceState } from "../session-state";

export function createSpaceViewActions(state: SpaceState, placement: SpacePlacement) {
  function focusItems(ids: readonly string[]) {
    const document = currentCanvasEditor()?.getDocument();
    if (!document) return;
    const tree = indexStructure(document.structure);
    const selected = structureSubtree(
      tree,
      ids.filter((id) => tree.items.has(id)),
    ).map((item) => item.id);
    state.canvasFocusIds.value = includeContainers(new Set(selected));
  }
  function focusNeighborhood(id: string, depth = 1) {
    state.canvasFocusIds.value = includeContainers(relationNeighborhood(state.relations.value, [id], depth));
  }
  function focusRelationPath(sourceId: string, targetId: string) {
    const path = findSemanticRelationPath(state.relations.value, sourceId, targetId);
    if (!path.length) return false;
    state.canvasFocusIds.value = includeContainers(new Set(path));
    return true;
  }
  async function openSceneView(id: string) {
    const scene = state.scenes.value.find((candidate) => candidate.id === id);
    if (!scene) return;
    state.relationKindFilter.value = [...scene.relationKinds];
    state.relationStatusFilter.value = [...scene.relationStatuses];
    state.savedQueryFilter.value = snapshotSpaceQueryFilter(scene.queryFilter);
    const dynamic = hasQueryFilter(scene.queryFilter)
      ? queryMatchingNodes(state.nodes.value, state.relations.value, scene.queryFilter).map((node) => node.id)
      : scene.focusNodeIds;
    if (dynamic.length) focusItems(dynamic);
    else state.canvasFocusIds.value = [];
    placement.requestViewport({ x: scene.viewportX, y: scene.viewportY, zoom: scene.viewportZoom });
    placement.announceScene(scene, "opened");
  }
  return {
    focusItems,
    focusSection: (id: string) => focusItems([id]),
    focusNeighborhood,
    focusRelationPath,
    clearCanvasFocus: () => {
      state.canvasFocusIds.value = [];
    },
    openSceneView,
  };
}
function includeContainers(ids: Set<string>) {
  const document = currentCanvasEditor()?.getDocument();
  if (document) {
    const tree = indexStructure(document.structure);
    for (const id of [...ids])
      if (tree.items.has(id)) for (const parent of structureAncestors(tree, id)) ids.add(parent.id);
  }
  return [...ids];
}
function hasQueryFilter(filter: SpaceSavedViewFilter) {
  return Object.values(filter).some((value) =>
    Array.isArray(value) ? value.length > 0 : value !== undefined && value !== "",
  );
}
