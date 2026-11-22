import type { SpacePlacement } from "../../canvas/navigation-requests";
import { findSemanticRelationPath, relationNeighborhood } from "../../knowledge/model/relation-graph";
import { createStructureIndex } from "../../model/structure/structure-index";
import { contentNodeIdsWithin } from "../../model/structure/structure-queries";
import {
  applySceneProjection,
  buildSceneViewProjection,
  clearSceneProjection,
} from "../../workspace/space-view-session";
import type { SpaceState } from "../session-state";

export function createSpaceViewActions(state: SpaceState, placement: SpacePlacement) {
  function focusSection(sectionId: string) {
    state.activeSceneId.value = undefined;
    state.canvasFocusIds.value = contentNodeIdsWithin(
      [sectionId],
      createStructureIndex(state.editSnapshot.value.structure),
    );
  }

  function focusNeighborhood(nodeId: string, depth = 1) {
    state.activeSceneId.value = undefined;
    const ids = relationNeighborhood(state.relations.value, [nodeId], depth);
    state.canvasFocusIds.value = [...ids];
  }

  function focusRelationPath(sourceId: string, targetId: string) {
    const path = findSemanticRelationPath(state.relations.value, sourceId, targetId);
    if (!path.length) return false;
    state.activeSceneId.value = undefined;
    const ids = new Set(path);
    state.canvasFocusIds.value = [...ids];
    return true;
  }

  async function openSceneView(sceneId: string) {
    const scene = state.scenes.value.find((candidate) => candidate.id === sceneId);
    if (!scene) return;
    applySceneProjection(
      state,
      buildSceneViewProjection(scene, {
        nodes: state.nodes.value,
        relations: state.relations.value,
        editSnapshot: state.editSnapshot.value,
      }),
    );
    placement.requestViewport({ x: scene.viewportX, y: scene.viewportY, zoom: scene.viewportZoom });
    placement.announceScene(scene, "opened");
  }

  return {
    focusSection,
    focusNeighborhood,
    focusRelationPath,
    clearCanvasFocus: () => clearSceneProjection(state),
    openSceneView,
  };
}
