import { assetIdentity } from "../media/asset-cache";
import type { ImageLibrary } from "../media/preview-images";
import type { VideoLibrary } from "../media/preview-video";
import { createFrameFeed, type FrameFeed } from "../media/video-frames";
import type { VideoAsset } from "../project";

interface FrameSourceOptions {
  videos: VideoLibrary;
  images: ImageLibrary;
  onChanged: () => void;
  onFailed: (error: unknown) => void;
}

/**
 * Resolves the picture of every visual asset: a decoded video frame from its own feed, or a still
 * image bitmap. Opening media and decoding stay asynchronous; until a picture exists the caller is
 * told nothing is available yet rather than being handed a placeholder.
 */
export function createFrameSources(options: FrameSourceOptions) {
  const feeds = new Map<string, FrameFeed>();
  const failed = new Set<string>();

  function report(key: string, error: unknown) {
    if (failed.has(key)) return;
    failed.add(key);
    options.onFailed(error);
  }

  function feedOf(asset: VideoAsset) {
    const key = assetIdentity(asset);
    const known = feeds.get(key);
    if (known) return known;
    const preview = options.videos.ready(asset);
    if (!preview) {
      if (!failed.has(key)) {
        void options.videos.load(asset).then(options.onChanged, (error: unknown) => report(key, error));
      }
      return undefined;
    }
    const feed = createFrameFeed(preview.sink, {
      onDecoded: options.onChanged,
      onFailed: (error) => report(key, error),
    });
    feeds.set(key, feed);
    return feed;
  }

  function bitmapOf(asset: VideoAsset) {
    const key = assetIdentity(asset);
    const known = options.images.ready(asset);
    if (known || failed.has(key)) return known;
    void options.images.load(asset).then(options.onChanged, (error: unknown) => report(key, error));
    return undefined;
  }

  return {
    resolve(asset: VideoAsset, sourceMs: number, playing: boolean) {
      if (asset.kind === "image") return bitmapOf(asset);
      if (asset.kind !== "video") return undefined;
      return feedOf(asset)?.frameAt(sourceMs / 1000, playing);
    },
    forget(asset: VideoAsset) {
      const key = assetIdentity(asset);
      feeds.get(key)?.dispose();
      feeds.delete(key);
      failed.delete(key);
      options.videos.forget(asset);
      options.images.forget(asset);
    },
    dispose() {
      for (const feed of feeds.values()) feed.dispose();
      feeds.clear();
      failed.clear();
    },
  };
}
