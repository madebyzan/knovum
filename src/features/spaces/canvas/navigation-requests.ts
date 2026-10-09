import { union } from "es-toolkit";
import type { Ref } from "vue";

import type { SpaceScene } from "../scenes/scene";
import type {
  CanvasZoomMode,
  RevealRequest,
  SceneCue,
  ViewportRequest,
  ZoomRequest,
} from "../session/workbench-state";

interface SpacePlacementOptions {
  insertionPoint: Ref<{ x: number; y: number }>;
  revealRequest: Ref<RevealRequest | undefined>;
  viewportRequest: Ref<ViewportRequest | undefined>;
  zoomRequest: Ref<ZoomRequest | undefined>;
  selectionClearNonce: Ref<number>;
  sceneCue: Ref<SceneCue | undefined>;
}

export function createSpacePlacement(options: SpacePlacementOptions) {
  const { insertionPoint, revealRequest, viewportRequest, zoomRequest, selectionClearNonce, sceneCue } =
    options;
  let insertOffset = 0;
  let navigationNonce = 0;
  function nextInsertionPoint() {
    const offset = (insertOffset++ % 6) * 20;
    return { x: insertionPoint.value.x + offset, y: insertionPoint.value.y + offset };
  }
  function requestReveal(targetIds: readonly string[], mode: RevealRequest["mode"] = "ensure-visible") {
    const pending =
      mode === "ensure-visible" && revealRequest.value?.mode === mode ? revealRequest.value.ids : [];
    const uniqueIds = union(pending, [...targetIds]);
    if (uniqueIds.length) revealRequest.value = { ids: uniqueIds, nonce: ++navigationNonce, mode };
  }
  function requestViewport(viewport: ViewportRequest["viewport"]) {
    viewportRequest.value = { viewport: { ...viewport }, nonce: ++navigationNonce };
  }
  function announceScene(scene: SpaceScene, action: SceneCue["action"]) {
    sceneCue.value = {
      sceneId: scene.id,
      title: scene.title,
      action,
      zoom: scene.viewportZoom,
      nonce: ++navigationNonce,
    };
  }
  function requestZoom(mode: CanvasZoomMode) {
    zoomRequest.value = { mode, nonce: ++navigationNonce };
  }
  function requestClearSelection() {
    selectionClearNonce.value += 1;
  }
  return {
    nextInsertionPoint,
    requestReveal,
    requestViewport,
    requestZoom,
    requestClearSelection,
    announceScene,
  };
}

export type SpacePlacement = ReturnType<typeof createSpacePlacement>;
