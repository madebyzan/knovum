import type { Ref } from "vue";
import { z } from "zod";

import type { RelationKind } from "../contracts/generated/RelationKind";
import type { RelationStatus } from "../contracts/generated/RelationStatus";
import { snapshotSpaceQueryFilter } from "../knowledge/model/query-filter";
import { queryMatchingNodes } from "../knowledge/model/relation-graph";
import type { SpaceSavedViewFilter } from "../knowledge/model/saved-view-filter";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex, type StructureIndex } from "../model/structure/structure-index";
import {
  contentNodeIdsWithin,
  reconcileGroupScopePath,
  sameGroupScopePath,
} from "../model/structure/structure-queries";
import type { SpaceScene } from "../scenes/scene";

export const DEFAULT_RELATION_STATUS_FILTER: readonly RelationStatus[] = ["active", "contested"];

const nonEmptyId = z.string().min(1);

export const savedSpaceViewSchema = z
  .object({
    groupScopePath: z.array(nonEmptyId).optional(),
    activeSceneId: nonEmptyId.optional(),
    outlineExpandedIds: z.array(nonEmptyId).optional(),
  })
  .strict();

export type SavedSpaceViewSession = z.infer<typeof savedSpaceViewSchema>;

/** Per-Space transient view context preserved across tab switches and startup restoration. */
export interface SpaceViewSession {
  groupScopePath: string[];
  activeSceneId: string | undefined;
  outlineExpandedIds: string[];
  canvasFocusIds: string[];
  relationKindFilter: RelationKind[];
  relationStatusFilter: RelationStatus[];
  savedQueryFilter: SpaceSavedViewFilter;
}

export interface SpaceViewRefs {
  groupScopePath: Ref<string[]>;
  activeSceneId: Ref<string | undefined>;
  outlineExpandedIds: Ref<string[]>;
  canvasFocusIds: Ref<string[]>;
  relationKindFilter: Ref<RelationKind[]>;
  relationStatusFilter: Ref<RelationStatus[]>;
  savedQueryFilter: Ref<SpaceSavedViewFilter>;
}

export interface SpaceViewContent {
  nodes: readonly SpaceCanvasNode[];
  scenes: readonly SpaceScene[];
  relations: readonly SpaceRelation[];
  editSnapshot: SpaceEditSnapshot;
}

type SceneProjectionKey =
  | "activeSceneId"
  | "canvasFocusIds"
  | "relationKindFilter"
  | "relationStatusFilter"
  | "savedQueryFilter";
type SceneViewProjection = Pick<SpaceViewSession, SceneProjectionKey>;
type SceneProjectionRefs = Pick<SpaceViewRefs, SceneProjectionKey>;
type SavedSpaceViewKey = "groupScopePath" | "activeSceneId" | "outlineExpandedIds";

export function defaultSpaceViewSession(): SpaceViewSession {
  return {
    groupScopePath: [],
    activeSceneId: undefined,
    outlineExpandedIds: [],
    canvasFocusIds: [],
    relationKindFilter: [],
    relationStatusFilter: [...DEFAULT_RELATION_STATUS_FILTER],
    savedQueryFilter: {},
  };
}

export function captureSpaceViewSession(source: SpaceViewRefs): SpaceViewSession {
  return {
    groupScopePath: [...source.groupScopePath.value],
    activeSceneId: source.activeSceneId.value,
    outlineExpandedIds: [...source.outlineExpandedIds.value],
    canvasFocusIds: [...source.canvasFocusIds.value],
    relationKindFilter: [...source.relationKindFilter.value],
    relationStatusFilter: [...source.relationStatusFilter.value],
    savedQueryFilter: snapshotSpaceQueryFilter(source.savedQueryFilter.value),
  };
}

export function applySpaceViewSession(target: SpaceViewRefs, view: SpaceViewSession): void {
  target.groupScopePath.value = [...view.groupScopePath];
  target.outlineExpandedIds.value = [...view.outlineExpandedIds];
  applySceneProjection(target, view);
}

export function captureSavedSpaceView(
  source: Pick<SpaceViewRefs, SavedSpaceViewKey>,
): SavedSpaceViewSession | undefined {
  return serializeSpaceViewSession({
    groupScopePath: source.groupScopePath.value,
    activeSceneId: source.activeSceneId.value,
    outlineExpandedIds: source.outlineExpandedIds.value,
  });
}

export function serializeSpaceViewSession(
  view: Pick<SpaceViewSession, SavedSpaceViewKey>,
): SavedSpaceViewSession | undefined {
  const entry: SavedSpaceViewSession = {};
  if (view.groupScopePath.length) entry.groupScopePath = [...view.groupScopePath];
  if (view.activeSceneId) entry.activeSceneId = view.activeSceneId;
  if (view.outlineExpandedIds.length) entry.outlineExpandedIds = [...view.outlineExpandedIds];
  return Object.keys(entry).length ? entry : undefined;
}

/** Prunes live Group scope and Outline disclosures against an updated Space structure snapshot. */
export function reconcileSpaceViewStructure(
  target: Pick<SpaceViewRefs, "groupScopePath" | "outlineExpandedIds">,
  structure: SpaceEditSnapshot["structure"],
): void {
  if (!target.groupScopePath.value.length && !target.outlineExpandedIds.value.length) return;
  const index = createStructureIndex(structure);
  if (target.groupScopePath.value.length) {
    const next = reconcileGroupScopePath(target.groupScopePath.value, index);
    if (!sameGroupScopePath(target.groupScopePath.value, next)) target.groupScopePath.value = next;
  }
  if (target.outlineExpandedIds.value.length) {
    const next = reconcileOutlineExpandedIds(target.outlineExpandedIds.value, index);
    if (next.length !== target.outlineExpandedIds.value.length) target.outlineExpandedIds.value = next;
  }
}

/** Revalidates a saved Space view against loaded Space content without modifying its camera. */
export function restoreSpaceViewSession(
  saved: SavedSpaceViewSession | undefined,
  content: SpaceViewContent,
): SpaceViewSession {
  const base = defaultSpaceViewSession();
  if (!saved) return base;
  const index = createStructureIndex(content.editSnapshot.structure);
  const groupScopePath = saved.groupScopePath ? reconcileGroupScopePath(saved.groupScopePath, index) : [];
  const outlineExpandedIds = reconcileOutlineExpandedIds(saved.outlineExpandedIds ?? [], index);
  const scene = saved.activeSceneId
    ? content.scenes.find((candidate) => candidate.id === saved.activeSceneId)
    : undefined;
  const sceneProjection = scene ? buildSceneViewProjection(scene, content) : undefined;
  return { ...base, ...sceneProjection, groupScopePath, outlineExpandedIds };
}

/** Computes the focus/filter projection of a Saved View without modifying the canvas camera. */
export function buildSceneViewProjection(
  scene: SpaceScene,
  content: Pick<SpaceViewContent, "nodes" | "relations" | "editSnapshot">,
): SceneViewProjection {
  const dynamicIds = hasQueryFilter(scene.queryFilter)
    ? queryMatchingNodes({
        nodes: content.nodes,
        relations: content.relations,
        filter: scene.queryFilter,
      }).map((node) => node.id)
    : contentNodeIdsWithin(scene.focusStructureNodeIds, createStructureIndex(content.editSnapshot.structure));
  return {
    activeSceneId: scene.id,
    canvasFocusIds: [...new Set(dynamicIds)],
    relationKindFilter: [...scene.relationKinds],
    relationStatusFilter: [...scene.relationStatuses],
    savedQueryFilter: snapshotSpaceQueryFilter(scene.queryFilter),
  };
}

export function applySceneProjection(target: SceneProjectionRefs, projection: SceneViewProjection): void {
  target.activeSceneId.value = projection.activeSceneId;
  target.canvasFocusIds.value = [...projection.canvasFocusIds];
  target.relationKindFilter.value = [...projection.relationKindFilter];
  target.relationStatusFilter.value = [...projection.relationStatusFilter];
  target.savedQueryFilter.value = snapshotSpaceQueryFilter(projection.savedQueryFilter);
}

export function clearSceneProjection(target: SceneProjectionRefs): void {
  applySceneProjection(target, defaultSpaceViewSession());
}

function reconcileOutlineExpandedIds(expandedIds: readonly string[], index: StructureIndex): string[] {
  return expandedIds.filter((id) => {
    const item = index.structureById.get(id);
    return item !== undefined && item.kind !== "content";
  });
}

function hasQueryFilter(filter: SpaceSavedViewFilter) {
  return Object.values(filter).some((value) =>
    Array.isArray(value) ? value.length > 0 : value !== undefined && value !== "",
  );
}
