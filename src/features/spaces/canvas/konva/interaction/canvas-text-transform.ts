import {
  CANVAS_TYPOGRAPHY_SCALE_MAX,
  CANVAS_TYPOGRAPHY_SCALE_MIN,
  type CanvasNodeAppearance,
  type CanvasNodeLayoutV1,
} from "../../contract/canvas-document";
export function scaledCanvasTextAppearance(layout: CanvasNodeLayoutV1, width: number): CanvasNodeAppearance {
  const scale = layout.appearance?.typographyScale ?? 1;
  return {
    ...layout.appearance,
    typographyScale: Math.min(
      CANVAS_TYPOGRAPHY_SCALE_MAX,
      Math.max(CANVAS_TYPOGRAPHY_SCALE_MIN, scale * (width / layout.frame.width)),
    ),
  };
}
