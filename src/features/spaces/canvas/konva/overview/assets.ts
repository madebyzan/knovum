import { warn as logWarn } from "@tauri-apps/plugin-log";
import pLimit from "p-limit";

import { spaceAssetSource, spaceCanvasOverviewSource } from "../../../assets/node-preview-source";
import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasBounds } from "../../canvas-snap-geometry";
import type { CanvasScene } from "../scene/canvas-scene";

const overviewDecodes = pLimit(4);

export async function loadCanvasOverviewImages(
  scene: CanvasScene,
  scale: number,
  viewport: Pick<CanvasBounds, "left" | "top" | "right" | "bottom">,
) {
  const candidates = scene.nodes.flatMap(({ layout, node }) => {
    const bounds = canvasFrameBounds(layout.frame);
    if (
      bounds.right < viewport.left ||
      bounds.left > viewport.right ||
      bounds.bottom < viewport.top ||
      bounds.top > viewport.bottom
    )
      return [];
    if (layout.frame.width * scale < 16 || layout.frame.height * scale < 16) return [];
    const source = layout.appearance?.background
      ? spaceAssetSource(layout.appearance.background)
      : spaceCanvasOverviewSource(node.data);
    return source ? [{ id: node.id, source }] : [];
  });
  const imageByNodeId = new Map<string, HTMLImageElement>();
  let unavailable = 0;
  await Promise.all(
    candidates.map((candidate) =>
      overviewDecodes(async () => {
        try {
          const image = new Image();
          image.crossOrigin = "anonymous";
          image.src = candidate.source;
          await image.decode();
          imageByNodeId.set(candidate.id, image);
        } catch {
          unavailable += 1;
        }
      }),
    ),
  );
  if (unavailable) {
    void logWarn(`Canvas overview could not load ${unavailable} preview image(s)`);
  }
  return imageByNodeId;
}
