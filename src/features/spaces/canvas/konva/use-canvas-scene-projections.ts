import { computed, type Ref } from "vue";

import { ancestorsOf, createStructureIndex } from "../../model/structure/structure-index";
import { isStructureNodeEffectivelyLocked } from "../../model/structure/structure-queries";
import type { WorldBounds } from "../../model/structure/world-bounds";
import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import { SECTION_HEADER_HEIGHT } from "../document/structure-scene";
import { projectStructureScene } from "../document/structure-scene-projection";
import type { CanvasDocumentV1 } from "../schema/document";
import { projectCanvasScene } from "./scene/scene-projection";

export function useCanvasSceneProjections(
  props: CanvasSurfaceProps,
  document: Readonly<Ref<CanvasDocumentV1>>,
  temporarilyExpandedSectionId: Readonly<Ref<string | undefined>>,
) {
  const structureIndex = computed(() => createStructureIndex(props.sceneInput.editSnapshot.structure));
  const lockedNodeIds = computed(() => {
    const index = structureIndex.value;
    return new Set(
      props.sceneInput.nodes.flatMap(({ id }) => {
        const structureNodeId = index.nodeIdToStructureNodeId.get(id);
        if (!structureNodeId) return [];
        return isStructureNodeEffectivelyLocked(structureNodeId, index) ? [id] : [];
      }),
    );
  });
  const structureScene = computed(() =>
    projectStructureScene({
      snapshot: props.sceneInput.editSnapshot,
      nodes: props.sceneInput.nodes,
      relations: props.sceneInput.relations,
      temporarilyExpandedSectionIds: temporarilyExpandedSectionId.value
        ? new Set([temporarilyExpandedSectionId.value])
        : new Set(),
    }),
  );
  const scene = computed(() =>
    projectCanvasScene({
      document: document.value,
      nodes: props.sceneInput.nodes,
      relations: props.sceneInput.relations,
      lockedNodeIds: lockedNodeIds.value,
      structure: structureScene.value,
    }),
  );
  const relationBounds = computed(
    () =>
      new Map(
        scene.value.relations.map(({ id, midpoint }) => [
          id,
          { x: midpoint.x, y: midpoint.y, width: 0, height: 0 },
        ]),
      ),
  );
  const structureBounds = computed(
    () =>
      new Map([
        ...structureScene.value.content.map(
          ({ bounds, structureNode }) => [structureNode.id, canvasBounds(bounds)] as const,
        ),
        ...structureScene.value.groups.map(({ bounds, group }) => [group.id, canvasBounds(bounds)] as const),
        ...structureScene.value.sections.map(
          ({ bounds, section, layout }) =>
            [
              section.id,
              canvasBounds(
                layout.collapsed && temporarilyExpandedSectionId.value !== section.id
                  ? { ...bounds, bottom: bounds.top + SECTION_HEADER_HEIGHT }
                  : bounds,
              ),
            ] as const,
        ),
      ]),
  );
  const groupScopeTitles = computed(() => {
    const index = structureIndex.value;
    return props.groupScopePath.map((structureNodeId) => {
      const group = index.structureById.get(structureNodeId);
      if (group?.kind !== "group") throw new Error(`Group scope is unavailable: ${structureNodeId}`);
      return group.title;
    });
  });
  function structureSelectionForNode(nodeId: string) {
    const index = structureIndex.value;
    const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
    if (!structureNodeId) throw new Error(`Content structure is unavailable: ${nodeId}`);
    const ancestors = ancestorsOf(structureNodeId, index);
    const scopeId = props.groupScopePath.at(-1);
    const scopeIndex = scopeId ? ancestors.findIndex(({ id }) => id === scopeId) : -1;
    return ancestors.slice(scopeIndex + 1).find(({ kind }) => kind === "group")?.id;
  }
  return {
    groupScopeTitles,
    lockedNodeIds,
    relationBounds,
    scene,
    structureBounds,
    structureScene,
    structureSelectionForNode,
  };
}

function canvasBounds(bounds: WorldBounds) {
  return {
    x: bounds.left,
    y: bounds.top,
    width: bounds.right - bounds.left,
    height: bounds.bottom - bounds.top,
  };
}
