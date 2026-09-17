import { sortBy, sumBy } from "es-toolkit";

import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../contract/canvas-document";
import type { CanvasAlignMode, CanvasDistributeMode } from "../contract/canvas-editor-port";

export function alignCanvasLayouts(
  layouts: readonly CanvasNodeLayoutV1[],
  mode: CanvasAlignMode,
): CanvasNodeFrameUpdate[] {
  if (layouts.length < 2) return [];
  const area = bounds(layouts);
  return layouts.map(({ nodeId, frame }) => ({ nodeId, frame: alignedFrame(frame, area, mode) }));
}

export function distributeCanvasLayouts(
  layouts: readonly CanvasNodeLayoutV1[],
  mode: CanvasDistributeMode,
): CanvasNodeFrameUpdate[] {
  if (layouts.length < 3) return [];
  const horizontal = mode === "horizontal";
  const ordered = sortBy(layouts, [({ frame }) => (horizontal ? frame.x : frame.y)]);
  const area = bounds(ordered);
  const occupied = sumBy(ordered, ({ frame }) => (horizontal ? frame.width : frame.height));
  const span = horizontal ? area.right - area.left : area.bottom - area.top;
  const gap = (span - occupied) / (ordered.length - 1);
  let cursor = horizontal ? area.left : area.top;
  return ordered.map(({ nodeId, frame }) => {
    const next = { ...frame, ...(horizontal ? { x: cursor } : { y: cursor }) };
    cursor += (horizontal ? frame.width : frame.height) + gap;
    return { nodeId, frame: next };
  });
}

export function fitSectionLayout(
  section: CanvasNodeLayoutV1,
  members: readonly CanvasNodeLayoutV1[],
): CanvasNodeFrameUpdate[] {
  if (!members.length) return [];
  const area = bounds(members);
  return [
    {
      nodeId: section.nodeId,
      frame: {
        ...section.frame,
        x: area.left - 32,
        y: area.top - 56,
        width: area.right - area.left + 64,
        height: area.bottom - area.top + 88,
      },
    },
  ];
}

export function canvasLayoutBounds(layouts: readonly CanvasNodeLayoutV1[]) {
  if (!layouts.length) return undefined;
  const area = bounds(layouts);
  return { x: area.left, y: area.top, width: area.right - area.left, height: area.bottom - area.top };
}

function bounds(layouts: readonly CanvasNodeLayoutV1[]) {
  const left = Math.min(...layouts.map(({ frame }) => frame.x));
  const top = Math.min(...layouts.map(({ frame }) => frame.y));
  const right = Math.max(...layouts.map(({ frame }) => frame.x + frame.width));
  const bottom = Math.max(...layouts.map(({ frame }) => frame.y + frame.height));
  return { left, top, right, bottom, centerX: (left + right) / 2, centerY: (top + bottom) / 2 };
}

function alignedFrame(
  frame: CanvasNodeLayoutV1["frame"],
  area: ReturnType<typeof bounds>,
  mode: CanvasAlignMode,
) {
  if (mode === "left") return { ...frame, x: area.left };
  if (mode === "right") return { ...frame, x: area.right - frame.width };
  if (mode === "center") return { ...frame, x: area.centerX - frame.width / 2 };
  if (mode === "top") return { ...frame, y: area.top };
  if (mode === "bottom") return { ...frame, y: area.bottom - frame.height };
  return { ...frame, y: area.centerY - frame.height / 2 };
}
