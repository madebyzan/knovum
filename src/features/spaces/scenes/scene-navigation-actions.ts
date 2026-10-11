import { nextTick, type Ref } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type { CanvasViewport } from "../canvas/contracts/canvas-editor-port";
import type { Space } from "../model/space";
import { useSpaceStore } from "../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem } from "../workspace/space-menu";
import type { SpaceScene } from "./scene";

export interface SceneNavigationContext {
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
  const store = useSpaceStore();
  return {
    createScene,
    openScene: (scene: SpaceScene) => store.openSceneView(scene.id),
    beginRename: (scene: SpaceScene) => beginRename(context, scene),
    finishRename,
    showSceneMenu,
  };

  async function createScene(thumbnailPath: string) {
    const spaceId = context.activeSpace.value?.id;
    const viewport = context.viewport.value;
    if (!spaceId || !viewport) return;
    const value =
      context.title.value.trim() ||
      t("spaces.scenes.defaultName", { index: context.scenes.value.length + 1 });
    const scene = await store.createScene(value, viewport, thumbnailPath);
    if (!scene) return;
    if (context.activeSpace.value?.id !== spaceId || scene.spaceId !== spaceId) return scene;
    context.title.value = "";
    store.announceScene(scene, "saved");
    await revealScene(context, scene.id);
    return scene;
  }

  function finishRename() {
    const id = context.editingId.value;
    const value = context.editingTitle.value.trim();
    if (id && value) void store.updateScene(id, { title: value });
    context.editingId.value = undefined;
  }

  async function updateScene(scene: SpaceScene) {
    const spaceId = context.activeSpace.value?.id;
    const viewport = context.viewport.value;
    if (!spaceId || scene.spaceId !== spaceId || !viewport) return;
    const updated = await store.updateScene(scene.id, { viewport, captureView: true });
    if (!updated || context.activeSpace.value?.id !== spaceId) return;
    store.announceScene(updated, "updated");
    await revealScene(context, updated.id);
  }

  async function duplicateScene(scene: SpaceScene) {
    const spaceId = context.activeSpace.value?.id;
    if (!spaceId || scene.spaceId !== spaceId) return;
    const duplicate = await store.duplicateScene(scene.id);
    if (!duplicate || context.activeSpace.value?.id !== spaceId) return;
    store.announceScene(duplicate, "saved");
    await revealScene(context, duplicate.id);
  }

  async function showSceneMenu(scene: SpaceScene) {
    const index = context.scenes.value.findIndex((candidate) => candidate.id === scene.id);
    await popupSpaceContextMenu([
      [spaceMenuItem(t("spaces.scenes.open"), () => void store.openSceneView(scene.id))],
      [
        spaceMenuItem(t("spaces.scenes.update"), () => void updateScene(scene)),
        spaceMenuItem(t("spaces.scenes.duplicate"), () => void duplicateScene(scene)),
      ],
      [
        spaceMenuItem(t("spaces.scenes.moveUp"), () => store.moveScene(scene.id, -1), {
          enabled: index > 0,
        }),
        spaceMenuItem(t("spaces.scenes.moveDown"), () => store.moveScene(scene.id, 1), {
          enabled: index >= 0 && index < context.scenes.value.length - 1,
        }),
      ],
      [spaceMenuItem(t("spaces.scenes.rename"), () => void beginRename(context, scene))],
      [spaceMenuItem(t("spaces.scenes.delete"), () => store.removeScene(scene.id))],
    ]);
  }
}

async function beginRename(context: SceneNavigationContext, scene: SpaceScene) {
  context.editingId.value = scene.id;
  context.editingTitle.value = scene.title;
  await nextTick();
  context.root.value?.querySelector<HTMLInputElement>("[data-scene-editor]")?.select();
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
