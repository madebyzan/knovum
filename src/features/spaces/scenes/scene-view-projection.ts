import type { Ref } from "vue";

import type { RelationKind } from "../contracts/generated/RelationKind";
import type { RelationStatus } from "../contracts/generated/RelationStatus";
import { snapshotSpaceQueryFilter } from "../knowledge/model/query-filter";
import { queryMatchingNodes } from "../knowledge/model/relation-graph";
import type { SpaceSavedViewFilter } from "../knowledge/model/saved-view-filter";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../model/structure/structure-index";
import { contentNodeIdsWithin } from "../model/structure/structure-queries";
import type { SpaceScene } from "./scene";

/** The focus and filter fields of the view session that a Saved View drives. */
export interface SceneViewProjection {
  activeSceneId: string | undefined;
  canvasFocusIds: string[];
  relationKindFilter: RelationKind[];
  relationStatusFilter: RelationStatus[];
  savedQueryFilter: SpaceSavedViewFilter;
}

export type SceneProjectionRefs = { [K in keyof SceneViewProjection]: Ref<SceneViewProjection[K]> };

export interface SceneProjectionContent {
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
  editSnapshot: SpaceEditSnapshot;
}

/** Computes the focus/filter projection of a Saved View without modifying the canvas camera. */
export function buildSceneViewProjection(
  scene: SpaceScene,
  content: SceneProjectionContent,
): SceneViewProjection {
  const { nodes, relations, editSnapshot } = content;
  const filter = scene.queryFilter;
  const dynamicIds = hasQueryFilter(filter)
    ? queryMatchingNodes({ nodes, relations, filter }).map((node) => node.id)
    : contentNodeIdsWithin(scene.focusStructureNodeIds, createStructureIndex(editSnapshot.structure));
  return {
    activeSceneId: scene.id,
    canvasFocusIds: [...new Set(dynamicIds)],
    relationKindFilter: [...scene.relationKinds],
    relationStatusFilter: [...scene.relationStatuses],
    savedQueryFilter: snapshotSpaceQueryFilter(filter),
  };
}

export function applySceneProjection(target: SceneProjectionRefs, projection: SceneViewProjection): void {
  target.activeSceneId.value = projection.activeSceneId;
  target.canvasFocusIds.value = [...projection.canvasFocusIds];
  target.relationKindFilter.value = [...projection.relationKindFilter];
  target.relationStatusFilter.value = [...projection.relationStatusFilter];
  target.savedQueryFilter.value = snapshotSpaceQueryFilter(projection.savedQueryFilter);
}

function hasQueryFilter(filter: SpaceSavedViewFilter): boolean {
  return Object.values(filter).some((value) =>
    Array.isArray(value) ? value.length > 0 : value !== undefined && value !== "",
  );
}
