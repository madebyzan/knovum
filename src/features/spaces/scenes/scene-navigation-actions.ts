import { nextTick, type Ref } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import type { CanvasViewport } from "../canvas/contracts/canvas-editor-port";
import type { Space } from "../model/space";
import type { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem } from "../workspace/space-menu";
import type { SpaceScene } from "./scene";

export interface SceneNavigationContext {
  store: Pick<
    ReturnType<typeof useSpaceStore>,
    | "openSceneView"
    | "createScene"
    | "announceScene"
    | "updateScene"
    | "duplicateScene"
    | "moveScene"
    | "removeScene"
  >;
  scenes: Ref<SpaceScene[]>;
  activeSpace: Ref<Space | undefined>;
  viewport: Ref<CanvasViewport | undefined>;
  title: Ref<string>;
  editingId: Ref<string | undefined>;
  editingTitle: Ref<string>;
  recentSceneId: Ref<string | undefined>;
  root: Ref<HTMLElement | undefined>;
  triggerRecentTimer: () => void;
  stopRecentTimer: () => void;
}

const t = i18n.global.t;

export function createSceneNavigationActions(context: SceneNavigationContext) {
  return {
    createScene: (thumbnailPath: string) => createScene(context, thumbnailPath),
    openScene: (scene: SpaceScene) => context.store.openSceneView(scene.id),
    beginRename: (scene: SpaceScene) => beginRename(context, scene),
    finishRename: () => finishRename(context),
    showSceneMenu: (scene: SpaceScene) => showSceneMenu(context, scene),
  };
}

async function createScene(context: SceneNavigationContext, thumbnailPath: string) {
  const spaceId = context.activeSpace.value?.id;
  const viewport = context.viewport.value;
  if (!spaceId || !viewport) return;
  const value =
    context.title.value.trim() || t("spaces.scenes.defaultName", { index: context.scenes.value.length + 1 });
  const scene = await context.store.createScene(value, viewport, thumbnailPath);
  if (!scene) return;
  if (context.activeSpace.value?.id !== spaceId || scene.spaceId !== spaceId) return scene;
  context.title.value = "";
  context.store.announceScene(scene, "saved");
  await revealScene(context, scene.id);
  return scene;
}

async function beginRename(context: SceneNavigationContext, scene: SpaceScene) {
  context.editingId.value = scene.id;
  context.editingTitle.value = scene.title;
  await nextTick();
  context.root.value?.querySelector<HTMLInputElement>("[data-scene-editor]")?.select();
}

function finishRename(context: SceneNavigationContext) {
  const id = context.editingId.value;
  const value = context.editingTitle.value.trim();
  if (id && value) void context.store.updateScene(id, { title: value });
  context.editingId.value = undefined;
}

async function updateScene(context: SceneNavigationContext, scene: SpaceScene) {
  const spaceId = context.activeSpace.value?.id;
  const viewport = context.viewport.value;
  if (!spaceId || scene.spaceId !== spaceId || !viewport) return;
  const updated = await context.store.updateScene(scene.id, { viewport, captureView: true });
  if (!updated || context.activeSpace.value?.id !== spaceId) return;
  context.store.announceScene(updated, "updated");
  await revealScene(context, updated.id);
}

async function duplicateScene(context: SceneNavigationContext, scene: SpaceScene) {
  const spaceId = context.activeSpace.value?.id;
  if (!spaceId || scene.spaceId !== spaceId) return;
  const duplicate = await context.store.duplicateScene(scene.id);
  if (!duplicate || context.activeSpace.value?.id !== spaceId) return;
  context.store.announceScene(duplicate, "saved");
  await revealScene(context, duplicate.id);
}

async function revealScene(context: SceneNavigationContext, sceneId: string) {
  context.recentSceneId.value = sceneId;
  context.stopRecentTimer();
  await nextTick();
  context.root.value
    ?.querySelector<HTMLElement>(`[data-scene-id="${CSS.escape(sceneId)}"]`)
    ?.scrollIntoView({ block: "nearest" });
  context.triggerRecentTimer();
}

async function showSceneMenu(context: SceneNavigationContext, scene: SpaceScene) {
  const index = context.scenes.value.findIndex((candidate) => candidate.id === scene.id);
  await popupSpaceContextMenu([
    [spaceMenuItem(t("spaces.scenes.open"), () => void context.store.openSceneView(scene.id))],
    [
      spaceMenuItem(t("spaces.scenes.update"), () => void updateScene(context, scene)),
      spaceMenuItem(t("spaces.scenes.duplicate"), () => void duplicateScene(context, scene)),
    ],
    [
      spaceMenuItem(t("spaces.scenes.moveUp"), () => context.store.moveScene(scene.id, -1), {
        enabled: index > 0,
      }),
      spaceMenuItem(t("spaces.scenes.moveDown"), () => context.store.moveScene(scene.id, 1), {
        enabled: index >= 0 && index < context.scenes.value.length - 1,
      }),
    ],
    [spaceMenuItem(t("spaces.scenes.rename"), () => void beginRename(context, scene))],
    [spaceMenuItem(t("spaces.scenes.delete"), () => context.store.removeScene(scene.id))],
  ]);
}
