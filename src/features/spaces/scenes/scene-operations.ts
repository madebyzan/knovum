import { pick } from "es-toolkit";

import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import type { SpaceScene } from "./scene";

function sceneInput(scene: SpaceScene) {
  return pick(scene, [
    "id",
    "spaceId",
    "title",
    "thumbnailPath",
    "viewportX",
    "viewportY",
    "viewportZoom",
    "orderIndex",
    "focusStructureNodeIds",
    "relationKinds",
    "relationStatuses",
    "queryFilter",
  ]);
}

export function createScene(scene: SpaceScene): Promise<void> {
  return invoke("plugin:spaces|create_scene", { scene: sceneInput(scene) });
}

export function updateScene(scene: SpaceScene): Promise<string[]> {
  return invoke("plugin:spaces|update_scene", { scene: sceneInput(scene) });
}

export function updateSceneOrders(spaceId: string, scenes: readonly SpaceScene[]): Promise<void> {
  if (!scenes.length) return Promise.resolve();
  return invoke("plugin:spaces|update_scene_orders", {
    spaceId,
    scenes: scenes.map(({ id, orderIndex }) => ({ id, orderIndex })),
  });
}

export function deleteScene(sceneId: string): Promise<string[]> {
  return invoke("plugin:spaces|delete_scene", { sceneId });
}
