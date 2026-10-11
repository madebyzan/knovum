import type { InjectionKey } from "vue";

export interface CanvasContentMeasurementRegistry {
  register(measure: () => void): () => void;
}

export const canvasContentMeasurementKey: InjectionKey<CanvasContentMeasurementRegistry> = Symbol(
  "canvas-content-measurement",
);
