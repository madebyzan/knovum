import type Konva from "konva";
import { computed, shallowRef, type Ref } from "vue";

import type { CanvasPoint, CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasCamera } from "./canvas-camera";

interface CameraPanOptions {
  /** Stops a running camera animation when a gesture takes over. */
  abortAnimation(): void;
  camera: CanvasCamera;
  navigationEnabled(): boolean;
  spacePressed: Readonly<Ref<boolean>>;
}

interface CameraPanSession {
  origin: CanvasPoint;
  pointerId: number;
  viewport: CanvasViewport;
}

function panButtonPressed(event: PointerEvent, handTool: boolean, spacePressed: boolean) {
  if (event.button === 1) return true;
  return event.button === 0 && (handTool || spacePressed);
}

/**
 * Pointer panning. Every pointer move writes the camera directly; the camera coalesces the writes into one
 * projected frame, so no gesture code schedules animation frames of its own.
 */
export function createCameraPan(options: CameraPanOptions) {
  const session = shallowRef<CameraPanSession>();

  function begin(event: Konva.KonvaEventObject<PointerEvent>, handTool: boolean) {
    if (!options.navigationEnabled()) return false;
    if (!panButtonPressed(event.evt, handTool, options.spacePressed.value)) return false;
    if (session.value) return true;
    const origin = event.target.getStage()?.getPointerPosition();
    if (!origin) return false;
    event.evt.preventDefault();
    options.abortAnimation();
    session.value = { origin, pointerId: event.evt.pointerId, viewport: options.camera.get() };
    return true;
  }

  function move(event: Konva.KonvaEventObject<PointerEvent>) {
    const active = session.value;
    if (!active || event.evt.pointerId !== active.pointerId) return;
    const pointer = event.target.getStage()?.getPointerPosition();
    if (!pointer) return;
    options.camera.set({
      x: active.viewport.x + pointer.x - active.origin.x,
      y: active.viewport.y + pointer.y - active.origin.y,
      zoom: active.viewport.zoom,
    });
  }

  function owns(event: PointerEvent | undefined) {
    return Boolean(session.value) && (!event || event.pointerId === session.value?.pointerId);
  }

  return {
    begin,
    move,
    owns,
    panning: computed(() => session.value !== undefined),
    release: () => (session.value = undefined),
  };
}
