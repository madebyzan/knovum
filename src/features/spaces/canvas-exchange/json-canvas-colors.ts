import type { CanvasNodeAppearance } from "../canvas/contract/canvas-document";

type CanvasFillPreset = NonNullable<CanvasNodeAppearance["fill"]>;

const FILL_BY_PRESET = {
  "1": "pink",
  "2": "pink",
  "3": "yellow",
  "4": "green",
  "5": "blue",
  "6": "gray",
} as const;
const PRESET_BY_FILL = { yellow: "3", green: "4", blue: "5", pink: "1", gray: "6" } as const;

export function canvasFillForJsonColor(color: string) {
  const fill = FILL_BY_PRESET[color as keyof typeof FILL_BY_PRESET];
  return fill ? { fill } : { customFill: color };
}

export function jsonColorForCanvasFill(fill: CanvasFillPreset) {
  return PRESET_BY_FILL[fill];
}
