import { maxBy, sortBy } from "es-toolkit";
import type { Ref } from "vue";

import { cleanupDeletedSpaceAssets } from "../assets/asset-operations";
import { snapshotSpaceQueryFilter } from "../knowledge/model/query-filter";
import type { SpaceSavedViewFilter } from "../knowledge/model/saved-view-filter";
import type { SpaceRelationKind, SpaceRelationStatus } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../model/structure/structure-index";
import type { SpaceScene } from "./scene";
import { deleteScene, createScene as persistScene, updateSceneOrders, updateScene } from "./scene-operations";
import type { SpaceViewport } from "./viewport";

interface SceneActionContext {
  activeSpaceId: Ref<string | undefined>;
  scenes: Ref<SpaceScene[]>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  focusNodeIds: Ref<string[]>;
  relationKinds: Ref<SpaceRelationKind[]>;
  relationStatuses: Ref<SpaceRelationStatus[]>;
  queryFilter: Ref<SpaceSavedViewFilter>;
}

export async function createSpaceScene(
  context: SceneActionContext,
  title: string,
  viewport: SpaceViewport,
  thumbnailPath: string,
) {
  const spaceId = context.activeSpaceId.value;
  if (!spaceId) return;
  const now = Date.now();
  const scene: SpaceScene = {
    id: crypto.randomUUID(),
    spaceId,
    title,
    thumbnailPath,
    orderIndex: (maxBy(context.scenes.value, (candidate) => candidate.orderIndex)?.orderIndex ?? 0) + 100,
    createdAt: now,
    updatedAt: now,
    viewportX: viewport.x,
    viewportY: viewport.y,
    viewportZoom: viewport.zoom,
    focusStructureNodeIds: currentFocusStructureNodeIds(context),
    relationKinds: [...context.relationKinds.value],
    relationStatuses: [...context.relationStatuses.value],
    queryFilter: snapshotSpaceQueryFilter(context.queryFilter.value),
  };
  await persistScene(scene);
  if (context.activeSpaceId.value === spaceId) context.scenes.value = [...context.scenes.value, scene];
  return scene;
}

export async function updateSpaceScene(
  context: SceneActionContext,
  id: string,
  patch: { title?: string; viewport?: SpaceViewport; thumbnailPath?: string; captureView?: boolean },
) {
  const scene = context.scenes.value.find((candidate) => candidate.id === id);
  if (!scene) return;
  const viewport = patch.viewport;
  const view = patch.captureView ? currentView(context) : {};
  const next: SpaceScene = {
    ...scene,
    ...(patch.title === undefined ? {} : { title: patch.title }),
    ...(patch.thumbnailPath === undefined ? {} : { thumbnailPath: patch.thumbnailPath }),
    ...(viewport ? { viewportX: viewport.x, viewportY: viewport.y, viewportZoom: viewport.zoom } : {}),
    ...view,
    updatedAt: Date.now(),
  };
  const obsoletePaths = await updateScene(next);
  await cleanupDeletedSpaceAssets(obsoletePaths);
  if (context.activeSpaceId.value === scene.spaceId) {
    context.scenes.value = context.scenes.value.map((candidate) => (candidate.id === id ? next : candidate));
  }
  return next;
}

export async function duplicateSpaceScene(context: SceneActionContext, id: string) {
  const source = context.scenes.value.find((scene) => scene.id === id);
  if (!source) return;
  const now = Date.now();
  const duplicate: SpaceScene = {
    ...source,
    id: crypto.randomUUID(),
    focusStructureNodeIds: [...source.focusStructureNodeIds],
    relationKinds: [...source.relationKinds],
    relationStatuses: [...source.relationStatuses],
    queryFilter: snapshotSpaceQueryFilter(source.queryFilter),
    orderIndex: (maxBy(context.scenes.value, (scene) => scene.orderIndex)?.orderIndex ?? 0) + 100,
    createdAt: now,
    updatedAt: now,
  };
  await persistScene(duplicate);
  if (context.activeSpaceId.value === source.spaceId) {
    context.scenes.value = [...context.scenes.value, duplicate];
  }
  return duplicate;
}

async function persistReorderedScenes(context: SceneActionContext, ordered: SpaceScene[]) {
  const now = Date.now();
  const next = ordered.map((scene, order) => reindexScene(scene, order, now));
  const spaceId = next[0]?.spaceId;
  if (!spaceId) return;
  await updateSceneOrders(spaceId, next);
  if (context.activeSpaceId.value === spaceId) context.scenes.value = next;
}

function reorderArray<T>(array: readonly T[], fromIndex: number, toIndex: number): T[] {
  const copy = [...array];
  const [moved] = copy.splice(fromIndex, 1);
  if (moved) copy.splice(toIndex, 0, moved);
  return copy;
}

export async function moveSpaceScene(context: SceneActionContext, id: string, direction: -1 | 1) {
  const ordered = sortBy(context.scenes.value, [(scene) => scene.orderIndex, (scene) => scene.createdAt]);
  const from = ordered.findIndex((scene) => scene.id === id);
  const to = from + direction;
  if (from < 0 || to < 0 || to >= ordered.length) return;
  await persistReorderedScenes(context, reorderArray(ordered, from, to));
}

export async function reorderSpaceScenes(context: SceneActionContext, sourceId: string, targetId: string) {
  const ordered = sortBy(context.scenes.value, [(scene) => scene.orderIndex, (scene) => scene.createdAt]);
  const from = ordered.findIndex((s) => s.id === sourceId);
  const to = ordered.findIndex((s) => s.id === targetId);
  if (from < 0 || to < 0 || from === to) return;
  await persistReorderedScenes(context, reorderArray(ordered, from, to));
}

function reindexScene(scene: SpaceScene, order: number, updatedAt: number): SpaceScene {
  return { ...scene, orderIndex: (order + 1) * 100, updatedAt };
}

export async function removeSpaceScene(context: SceneActionContext, id: string) {
  const scene = context.scenes.value.find((candidate) => candidate.id === id);
  if (!scene) return;
  const obsoletePaths = await deleteScene(id);
  await cleanupDeletedSpaceAssets(obsoletePaths);
  if (context.activeSpaceId.value === scene.spaceId) {
    context.scenes.value = context.scenes.value.filter((candidate) => candidate.id !== id);
  }
}

function currentView(context: SceneActionContext) {
  return {
    focusStructureNodeIds: currentFocusStructureNodeIds(context),
    relationKinds: [...context.relationKinds.value],
    relationStatuses: [...context.relationStatuses.value],
    queryFilter: snapshotSpaceQueryFilter(context.queryFilter.value),
  };
}

function currentFocusStructureNodeIds(context: SceneActionContext) {
  const index = createStructureIndex(context.editSnapshot.value.structure);
  return context.focusNodeIds.value.map((nodeId) => {
    const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
    if (!structureNodeId) throw new Error(`Saved View focus node has no structure node: ${nodeId}`);
    return structureNodeId;
  });
}
