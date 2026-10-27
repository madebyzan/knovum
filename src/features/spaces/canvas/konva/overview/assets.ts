import { warn as logWarn } from "@tauri-apps/plugin-log";
import pLimit from "p-limit";

import { spaceAssetSource, spaceCanvasOverviewSource } from "../../../assets/node-preview-source";
import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasBounds } from "../../canvas-snap-geometry";
import type { CanvasScene, CanvasSceneNode } from "../scene/canvas-scene";

/** The longest side an overview image is kept at: overviews draw nodes small, never at full detail. */
const OVERVIEW_IMAGE_EDGE = 384;
/**
 * Decoded overview images kept for reuse; the least recently used leaves the cache first. Leaving the
 * cache only drops its reference: an overview still drawing the bitmap keeps it alive until it is gone.
 */
const OVERVIEW_IMAGE_CACHE_SIZE = 400;

const overviewDecodes = pLimit(4);
const imageBySource = new Map<string, Promise<ImageBitmap>>();

/** The image a node's overview shows, if it has one. */
export function canvasOverviewImageSource({ layout, node }: CanvasSceneNode) {
  return layout.appearance?.background
    ? spaceAssetSource(layout.appearance.background)
    : spaceCanvasOverviewSource(node.data) || undefined;
}

/**
 * Decodes an overview image once, downscaled to overview size, so every overview of the same
 * source — the live canvas and Space covers alike — shares one small bitmap.
 */
export function loadCanvasOverviewImage(source: string): Promise<ImageBitmap> {
  const cached = imageBySource.get(source);
  if (cached) {
    imageBySource.delete(source);
    imageBySource.set(source, cached);
    return cached;
  }
  const loading = overviewDecodes(() => decodeOverviewImage(source));
  imageBySource.set(source, loading);
  loading.catch(() => imageBySource.delete(source));
  for (const stale of imageBySource.keys()) {
    if (imageBySource.size <= OVERVIEW_IMAGE_CACHE_SIZE) break;
    imageBySource.delete(stale);
  }
  return loading;
}

async function decodeOverviewImage(source: string) {
  const image = new Image();
  image.crossOrigin = "anonymous";
  image.src = source;
  await image.decode();
  const scale = Math.min(1, OVERVIEW_IMAGE_EDGE / Math.max(image.naturalWidth, image.naturalHeight, 1));
  return createImageBitmap(image, {
    resizeWidth: Math.max(1, Math.round(image.naturalWidth * scale)),
    resizeHeight: Math.max(1, Math.round(image.naturalHeight * scale)),
    resizeQuality: "medium",
  });
}

export async function loadCanvasOverviewImages(
  scene: CanvasScene,
  scale: number,
  viewport: Pick<CanvasBounds, "left" | "top" | "right" | "bottom">,
) {
  const candidates = scene.nodes.flatMap((sceneNode) => {
    const { frame } = sceneNode.layout;
    const bounds = canvasFrameBounds(frame);
    if (
      bounds.right < viewport.left ||
      bounds.left > viewport.right ||
      bounds.bottom < viewport.top ||
      bounds.top > viewport.bottom
    )
      return [];
    if (frame.width * scale < 16 || frame.height * scale < 16) return [];
    const source = canvasOverviewImageSource(sceneNode);
    return source ? [{ id: sceneNode.node.id, source }] : [];
  });
  const imageByNodeId = new Map<string, ImageBitmap>();
  let unavailable = 0;
  await Promise.all(
    candidates.map(async (candidate) => {
      try {
        imageByNodeId.set(candidate.id, await loadCanvasOverviewImage(candidate.source));
      } catch {
        unavailable += 1;
      }
    }),
  );
  if (unavailable) {
    void logWarn(`Canvas overview could not load ${unavailable} preview image(s)`);
  }
  return imageByNodeId;
}
