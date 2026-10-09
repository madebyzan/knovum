import { onBeforeUnmount, provide } from "vue";

import { registerSpaceSessionParticipant } from "../../workspace/session-participants";
import type { CanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import { canvasContentMeasurementKey } from "./canvas-content-measurement";

export function useCanvasContentLayoutSession(coordinator: CanvasContentLayoutCoordinator) {
  provide(canvasContentMeasurementKey, coordinator.measurements);
  const detach = registerSpaceSessionParticipant({
    commit: coordinator.settle,
    phase: "layout",
  });
  onBeforeUnmount(detach);
}
