import { warn as logWarn } from "@tauri-apps/plugin-log";

import { spaceAssetSource, spaceCanvasOverviewSource } from "../../../assets/node-preview-source";
import type { CanvasScene } from "../scene/scene-projection";

export async function loadCanvasOverviewImages(scene: CanvasScene, scale: number) {
  const candidates = scene.nodes.flatMap(({ layout, node }) => {
    if (layout.frame.width * scale < 16 || layout.frame.height * scale < 16) return [];
    const source = layout.appearance?.background
      ? spaceAssetSource(layout.appearance.background)
      : spaceCanvasOverviewSource(node.data);
    return source ? [{ id: node.id, source }] : [];
  });
  const images = new Map<string, HTMLImageElement>();
  let next = 0;
  let unavailable = 0;
  async function loadNext(): Promise<void> {
    const candidate = candidates[next++];
    if (!candidate) return;
    try {
      const image = new Image();
      image.crossOrigin = "anonymous";
      image.src = candidate.source;
      await image.decode();
      images.set(candidate.id, image);
    } catch {
      unavailable += 1;
    }
    await loadNext();
  }
  await Promise.all(Array.from({ length: Math.min(4, candidates.length) }, loadNext));
  if (unavailable) {
    void logWarn(`Canvas overview could not load ${unavailable} preview image(s)`);
  }
  return images;
}
