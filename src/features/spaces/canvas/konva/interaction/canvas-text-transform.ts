import { clamp } from "es-toolkit";

import {
  CONTENT_TYPOGRAPHY_SCALE_MAX,
  CONTENT_TYPOGRAPHY_SCALE_MIN,
} from "../../../model/structure/schema/content-appearance";
import type { CanvasNodeAppearance, CanvasNodeLayoutV1 } from "../../schema/document";

export function scaledCanvasTextAppearance(layout: CanvasNodeLayoutV1, width: number): CanvasNodeAppearance {
  const scale = layout.appearance?.typographyScale ?? 1;
  return {
    ...layout.appearance,
    typographyScale: clamp(
      scale * (width / layout.frame.width),
      CONTENT_TYPOGRAPHY_SCALE_MIN,
      CONTENT_TYPOGRAPHY_SCALE_MAX,
    ),
  };
}
