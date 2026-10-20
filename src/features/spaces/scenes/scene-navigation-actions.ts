import type { Ref } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type { CanvasViewport } from "../canvas/contracts/canvas-editor-port";
import type { Space } from "../model/space";
import { useSpaceStore } from "../session/space-store";
import type { SpaceScene } from "./scene";

export interface SceneNavigationContext {
  scenes: Ref<SpaceScene[]>;
  activeSpace: Ref<Space | undefined>;
  viewport: Ref<CanvasViewport | undefined>;
  recentSceneId: Ref<string | undefined>;
  triggerRecentTimer: () => void;
  stopRecentTimer: () => void;
}

const t = i18n.global.t;

export function createSceneNavigationActions(context: SceneNavigationContext) {
  const store = useSpaceStore();
  return {
    createScene,
    openScene: (scene: SpaceScene) => store.openSceneView(scene.id),
  };

  async function createScene(thumbnailPath: string) {
    const spaceId = context.activeSpace.value?.id;
    const viewport = context.viewport.value;
    if (!spaceId || !viewport) return;
    const title = t("spaces.scenes.defaultName", { index: context.scenes.value.length + 1 });
    const scene = await store.createScene(title, viewport, thumbnailPath);
    if (!scene) return;
    if (context.activeSpace.value?.id !== spaceId || scene.spaceId !== spaceId) return scene;
    store.announceScene(scene, "saved");
    markRecent(context, scene.id);
    return scene;
  }
}

function markRecent(context: SceneNavigationContext, sceneId: string) {
  context.stopRecentTimer();
  context.recentSceneId.value = sceneId;
  context.triggerRecentTimer();
}
