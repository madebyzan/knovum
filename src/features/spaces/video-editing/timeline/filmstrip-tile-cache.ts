export interface FilmstripTile {
  image: CanvasImageSource;
  exact: boolean;
}

function releaseCanvasBacking(image: CanvasImageSource) {
  if (
    (typeof HTMLCanvasElement !== "undefined" && image instanceof HTMLCanvasElement) ||
    (typeof OffscreenCanvas !== "undefined" && image instanceof OffscreenCanvas)
  ) {
    image.width = 0;
    image.height = 0;
  }
}

/**
 * Bounded LRU store for decoded filmstrip tiles that reference-counts shared keyframe canvases so
 * evicting one slot never zeroes a backing store still referenced by sibling slots.
 */
export function createFilmstripTileStore(capacity: number) {
  const tiles = new Map<string, FilmstripTile>();
  const canvasRefs = new Map<CanvasImageSource, number>();

  function retain(image: CanvasImageSource) {
    canvasRefs.set(image, (canvasRefs.get(image) ?? 0) + 1);
  }

  function release(tile: FilmstripTile | undefined) {
    if (!tile) return;
    const remaining = (canvasRefs.get(tile.image) ?? 0) - 1;
    if (remaining > 0) {
      canvasRefs.set(tile.image, remaining);
      return;
    }
    canvasRefs.delete(tile.image);
    releaseCanvasBacking(tile.image);
  }

  function keep(key: string, tile: FilmstripTile) {
    const existing = tiles.get(key);
    if (existing?.exact && !tile.exact) return;
    if (existing) {
      tiles.delete(key);
      if (existing.image !== tile.image) {
        retain(tile.image);
        release(existing);
      }
    } else {
      retain(tile.image);
    }
    tiles.set(key, tile);
    while (tiles.size > capacity) {
      const oldestKey = tiles.keys().next().value!;
      const oldest = tiles.get(oldestKey);
      tiles.delete(oldestKey);
      release(oldest);
    }
  }

  return {
    get: (key: string) => tiles.get(key),
    has: (key: string) => tiles.has(key),
    keep,
    clear() {
      for (const tile of tiles.values()) release(tile);
      tiles.clear();
      canvasRefs.clear();
    },
  };
}
