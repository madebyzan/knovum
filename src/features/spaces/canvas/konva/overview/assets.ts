import { warn as logWarn } from "@tauri-apps/plugin-log";
import pLimit from "p-limit";

import { spaceAssetSource, spaceCanvasOverviewSource } from "../../../assets/node-preview-source";
import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasBounds } from "../../canvas-snap-geometry";
import type { CanvasScene, CanvasSceneNode } from "../scene/canvas-scene";

/** The longest side an overview image is kept at: overviews draw nodes small, never at full detail. */
const OVERVIEW_IMAGE_EDGE = 384;
/**
 * Decoded overview images kept for reuse; the least recently used leaves the cache first.
 * Evicted bitmaps close their GPU backing store as soon as no live overview node references them.
 */
const OVERVIEW_IMAGE_CACHE_SIZE = 160;

interface OverviewImageEntry {
  promise: Promise<ImageBitmap>;
  bitmap?: ImageBitmap | undefined;
  refs: number;
  evicted: boolean;
}

const overviewDecodes = pLimit(4);
const entriesBySource = new Map<string, OverviewImageEntry>();
const entryByBitmap = new WeakMap<ImageBitmap, OverviewImageEntry>();

function maybeCloseEntry(entry: OverviewImageEntry) {
  if (!entry.evicted || entry.refs > 0 || !entry.bitmap) return;
  entry.bitmap.close();
  entry.bitmap = undefined;
}

/** Releases a caller's hold on an overview bitmap, closing it once evicted and unreferenced. */
export function releaseCanvasOverviewImage(bitmap: ImageBitmap | undefined): void {
  if (!bitmap) return;
  const entry = entryByBitmap.get(bitmap);
  if (!entry) return;
  entry.refs = Math.max(0, entry.refs - 1);
  maybeCloseEntry(entry);
}

/** The image a node's overview shows, if it has one. */
export function canvasOverviewImageSource({ layout, node }: CanvasSceneNode) {
  return layout.appearance?.background
    ? spaceAssetSource(layout.appearance.background)
    : spaceCanvasOverviewSource(node.data) || undefined;
}

function retainEntry(entry: OverviewImageEntry): Promise<ImageBitmap> {
  entry.refs += 1;
  return entry.promise.catch((error: unknown) => {
    entry.refs = Math.max(0, entry.refs - 1);
    maybeCloseEntry(entry);
    throw error;
  });
}

/**
 * Decodes an overview image once, downscaled to overview size, and returns a retained bitmap
 * owned by the caller until `releaseCanvasOverviewImage` is called.
 */
export function loadCanvasOverviewImage(source: string): Promise<ImageBitmap> {
  const cached = entriesBySource.get(source);
  if (cached) {
    entriesBySource.delete(source);
    entriesBySource.set(source, cached);
    return retainEntry(cached);
  }
  const entry: OverviewImageEntry = {
    promise: Promise.resolve() as unknown as Promise<ImageBitmap>,
    refs: 0,
    evicted: false,
  };
  entry.promise = overviewDecodes(() => decodeOverviewImage(source)).then((bitmap) => {
    entry.bitmap = bitmap;
    entryByBitmap.set(bitmap, entry);
    return bitmap;
  });
  entriesBySource.set(source, entry);
  entry.promise.catch(() => {
    if (entriesBySource.get(source) === entry) entriesBySource.delete(source);
  });
  for (const [staleKey, staleEntry] of entriesBySource) {
    if (entriesBySource.size <= OVERVIEW_IMAGE_CACHE_SIZE) break;
    entriesBySource.delete(staleKey);
    staleEntry.evicted = true;
    maybeCloseEntry(staleEntry);
  }
  return retainEntry(entry);
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
