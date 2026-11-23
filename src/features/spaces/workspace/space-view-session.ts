import type { Ref } from "vue";
import { z } from "zod";

import type { CanvasBackgroundPattern } from "../../../shared/contracts/generated/CanvasBackgroundPattern";
import { canvasBackgroundPatterns } from "../../../shared/contracts/generated/contract-values";
import type { RelationKind } from "../contracts/generated/RelationKind";
import type { RelationStatus } from "../contracts/generated/RelationStatus";
import { snapshotSpaceQueryFilter } from "../knowledge/model/query-filter";
import type { SpaceSavedViewFilter } from "../knowledge/model/saved-view-filter";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex, type StructureIndex } from "../model/structure/structure-index";
import { reconcileGroupScopePath, sameGroupScopePath } from "../model/structure/structure-queries";
import {
  applySourceReaderSession,
  captureSourceReaderSession,
  defaultSourceReaderSession,
  restoreSavedReaderState,
  serializeSavedReaderState,
  SOURCE_READER_WIDTH_MAX,
  SOURCE_READER_WIDTH_MIN,
  type SourceReaderSession,
} from "../reading/source-reader-target";
import type { SpaceScene } from "../scenes/scene";
import {
  applySceneProjection,
  buildSceneViewProjection,
  type SceneProjectionContent,
  type SceneProjectionRefs,
} from "../scenes/scene-view-projection";

export const DEFAULT_RELATION_STATUS_FILTER: readonly RelationStatus[] = ["active", "contested"];
const nonEmptyId = z.string().min(1);
const savedReaderTargetSchema = z.object({ nodeId: nonEmptyId, pane: z.enum(["read", "info"]) }).strict();

export const savedSpaceViewSchema = z
  .object({
    groupScopePath: z.array(nonEmptyId).optional(),
    activeSceneId: nonEmptyId.optional(),
    outlineExpandedIds: z.array(nonEmptyId).optional(),
    readingTarget: savedReaderTargetSchema.optional(),
    readerWidth: z.number().min(SOURCE_READER_WIDTH_MIN).max(SOURCE_READER_WIDTH_MAX).optional(),
    readerFocusMode: z.boolean().optional(),
    backgroundPattern: z.enum(canvasBackgroundPatterns).optional(),
  })
  .strict();

export type SavedSpaceViewSession = z.infer<typeof savedSpaceViewSchema>;

/** Per-Space transient view context preserved across tab switches and startup restoration. */
export interface SpaceViewSession extends SourceReaderSession {
  groupScopePath: string[];
  activeSceneId: string | undefined;
  outlineExpandedIds: string[];
  canvasFocusIds: string[];
  relationKindFilter: RelationKind[];
  relationStatusFilter: RelationStatus[];
  savedQueryFilter: SpaceSavedViewFilter;
  backgroundPattern: CanvasBackgroundPattern | undefined;
}

export type SpaceViewRefs = { [K in keyof SpaceViewSession]: Ref<SpaceViewSession[K]> };

export interface SpaceViewContent extends SceneProjectionContent {
  scenes: readonly SpaceScene[];
}

export type SavedSpaceViewKey = keyof SavedSpaceViewSession;

export function defaultSpaceViewSession(): SpaceViewSession {
  return {
    groupScopePath: [],
    activeSceneId: undefined,
    outlineExpandedIds: [],
    canvasFocusIds: [],
    relationKindFilter: [],
    relationStatusFilter: [...DEFAULT_RELATION_STATUS_FILTER],
    savedQueryFilter: {},
    backgroundPattern: undefined,
    ...defaultSourceReaderSession(),
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
    backgroundPattern: source.backgroundPattern.value,
    ...captureSourceReaderSession(source),
  };
}

export function applySpaceViewSession(target: SpaceViewRefs, view: SpaceViewSession): void {
  target.groupScopePath.value = [...view.groupScopePath];
  target.outlineExpandedIds.value = [...view.outlineExpandedIds];
  target.backgroundPattern.value = view.backgroundPattern;
  applySourceReaderSession(target, view);
  applySceneProjection(target, view);
}

export function captureSavedSpaceView(
  source: Pick<SpaceViewRefs, SavedSpaceViewKey>,
): SavedSpaceViewSession | undefined {
  return serializeSpaceViewSession({
    groupScopePath: source.groupScopePath.value,
    activeSceneId: source.activeSceneId.value,
    outlineExpandedIds: source.outlineExpandedIds.value,
    readingTarget: source.readingTarget.value,
    readerWidth: source.readerWidth.value,
    readerFocusMode: source.readerFocusMode.value,
    backgroundPattern: source.backgroundPattern.value,
  });
}

export function serializeSpaceViewSession(
  view: Pick<SpaceViewSession, SavedSpaceViewKey>,
): SavedSpaceViewSession | undefined {
  const entry: SavedSpaceViewSession = { ...serializeSavedReaderState(view) };
  if (view.groupScopePath.length) entry.groupScopePath = [...view.groupScopePath];
  if (view.activeSceneId) entry.activeSceneId = view.activeSceneId;
  if (view.outlineExpandedIds.length) entry.outlineExpandedIds = [...view.outlineExpandedIds];
  if (view.backgroundPattern) entry.backgroundPattern = view.backgroundPattern;
  return Object.keys(entry).length ? entry : undefined;
}

/** Prunes live Group scope and Outline disclosures against an updated Space structure snapshot. */
export function reconcileSpaceViewStructure(
  target: Pick<SpaceViewRefs, "groupScopePath" | "outlineExpandedIds">,
  structure: SpaceEditSnapshot["structure"],
): void {
  if (!target.groupScopePath.value.length && !target.outlineExpandedIds.value.length) return;
  const index = createStructureIndex(structure);
  const nextScope = reconcileGroupScopePath(target.groupScopePath.value, index);
  if (!sameGroupScopePath(target.groupScopePath.value, nextScope)) target.groupScopePath.value = nextScope;
  const nextExpanded = reconcileOutlineExpandedIds(target.outlineExpandedIds.value, index);
  if (nextExpanded.length !== target.outlineExpandedIds.value.length)
    target.outlineExpandedIds.value = nextExpanded;
}

/** Revalidates a saved Space view against loaded Space content without modifying its camera. */
export function restoreSpaceViewSession(
  saved: SavedSpaceViewSession | undefined,
  content: SpaceViewContent,
): SpaceViewSession {
  const base = defaultSpaceViewSession();
  if (!saved) return base;
  const index = createStructureIndex(content.editSnapshot.structure);
  const scene = saved.activeSceneId
    ? content.scenes.find((candidate) => candidate.id === saved.activeSceneId)
    : undefined;
  return {
    ...base,
    ...(scene ? buildSceneViewProjection(scene, content) : undefined),
    ...restoreSavedReaderState(saved, content.nodes),
    backgroundPattern: saved.backgroundPattern,
    groupScopePath: saved.groupScopePath ? reconcileGroupScopePath(saved.groupScopePath, index) : [],
    outlineExpandedIds: reconcileOutlineExpandedIds(saved.outlineExpandedIds ?? [], index),
  };
}

export function clearSceneProjection(target: SceneProjectionRefs): void {
  applySceneProjection(target, defaultSpaceViewSession());
}

function reconcileOutlineExpandedIds(ids: readonly string[], index: StructureIndex): string[] {
  return ids.filter((id) => ["section", "group"].includes(index.structureById.get(id)?.kind ?? ""));
}
