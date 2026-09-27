import { clamp } from "es-toolkit";

import {
  CANVAS_TYPOGRAPHY_SCALE_MAX,
  CANVAS_TYPOGRAPHY_SCALE_MIN,
  type CanvasNodeAppearance,
  type CanvasNodeLayoutV1,
} from "../../schema/document";
export function scaledCanvasTextAppearance(layout: CanvasNodeLayoutV1, width: number): CanvasNodeAppearance {
  const scale = layout.appearance?.typographyScale ?? 1;
  return {
    ...layout.appearance,
    typographyScale: clamp(
      scale * (width / layout.frame.width),
      CANVAS_TYPOGRAPHY_SCALE_MIN,
      CANVAS_TYPOGRAPHY_SCALE_MAX,
    ),
  };
}
