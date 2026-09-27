import type { CanvasNodeAppearance } from "../canvas/schema/document";

type CanvasFillPreset = NonNullable<CanvasNodeAppearance["fill"]>;

const PRESET_BY_FILL = { yellow: "3", green: "4", blue: "5", pink: "1", gray: "6" } as const;

export function jsonColorForCanvasFill(fill: CanvasFillPreset) {
  return PRESET_BY_FILL[fill];
}
