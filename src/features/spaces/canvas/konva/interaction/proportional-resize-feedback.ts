import type { CanvasNodeFrame } from "../../schema/document";
import type { CanvasGestureFeedback, CanvasSnapGuide } from "./canvas-snapping";

const MEASUREMENT_EXTENSION_PX = 120;
const BADGE_OFFSET_PX = 12;

export function projectResizeFeedback(
  frame: CanvasNodeFrame,
  snapGuides: readonly CanvasSnapGuide[],
  zoom: number,
  proportionalAnchor?: string,
): CanvasGestureFeedback {
  const measurement = proportionalAnchor
    ? proportionalResizeMeasurement(frame, proportionalAnchor, MEASUREMENT_EXTENSION_PX / zoom)
    : undefined;
  return {
    guides: [...snapGuides, ...(measurement ? [measurement.guide] : [])],
    resize: {
      width: frame.width,
      height: frame.height,
      x: measurement?.badge.x ?? frame.x + frame.width / 2,
      y: measurement?.badge.y ?? frame.y + frame.height + BADGE_OFFSET_PX / zoom,
    },
  };
}

function proportionalResizeMeasurement(
  frame: CanvasNodeFrame,
  activeAnchor: string,
  extension: number,
): { badge: { x: number; y: number }; guide: CanvasSnapGuide } | undefined {
  const corners = {
    "top-left": [
      { x: frame.x + frame.width, y: frame.y + frame.height },
      { x: frame.x, y: frame.y },
    ],
    "top-right": [
      { x: frame.x, y: frame.y + frame.height },
      { x: frame.x + frame.width, y: frame.y },
    ],
    "bottom-left": [
      { x: frame.x + frame.width, y: frame.y },
      { x: frame.x, y: frame.y + frame.height },
    ],
    "bottom-right": [
      { x: frame.x, y: frame.y },
      { x: frame.x + frame.width, y: frame.y + frame.height },
    ],
  } as const;
  const diagonal = corners[activeAnchor as keyof typeof corners];
  if (!diagonal) return undefined;
  const [origin, handle] = diagonal;
  const length = Math.hypot(handle.x - origin.x, handle.y - origin.y);
  const unit = { x: (handle.x - origin.x) / length, y: (handle.y - origin.y) / length };
  return {
    badge: {
      x: handle.x + unit.x * (extension / 10),
      y: handle.y + unit.y * (extension / 10),
    },
    guide: {
      kind: "measurement",
      points: [
        origin.x - unit.x * extension,
        origin.y - unit.y * extension,
        handle.x + unit.x * extension,
        handle.y + unit.y * extension,
      ],
    },
  };
}
