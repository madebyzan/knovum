import { assetIdentity } from "../media/asset-cache";
import type { ImageLibrary } from "../media/preview-images";
import type { VideoLibrary } from "../media/preview-video";
import { createFrameFeed, type FrameFeed } from "../media/video-frames";
import type { VideoAsset, VideoEditProject } from "../project";

interface FrameSourceOptions {
  videos: VideoLibrary;
  images: ImageLibrary;
  onChanged: () => void;
  onFailed: (error: unknown) => void;
}

function feedKey(asset: VideoAsset, slotKey?: string) {
  const assetKey = assetIdentity(asset);
  return slotKey ? `${assetKey}::${slotKey}` : assetKey;
}

/**
 * Resolves the picture of every visual asset: a hardware-decoded `<video>` feed per clip slot, or
 * a still image bitmap. Each clip slot owns an independent feed so cuts and transitions between
 * split clips of the same source asset stay pre-warmed and never thrash a single decoder.
 */
export function createFrameSources(options: FrameSourceOptions) {
  const feedByKey = new Map<string, FrameFeed>();
  const failed = new Set<string>();

  function report(key: string, error: unknown) {
    if (failed.has(key)) return;
    failed.add(key);
    options.onFailed(error);
  }

  function feedOf(asset: VideoAsset, slotKey?: string) {
    const assetKey = assetIdentity(asset);
    const key = feedKey(asset, slotKey);
    const known = feedByKey.get(key);
    if (known) return known;
    const preview = options.videos.ready(asset);
    if (!preview) {
      if (!failed.has(assetKey)) {
        void options.videos.load(asset).then(options.onChanged, (error: unknown) => report(assetKey, error));
      }
      return undefined;
    }
    const feed = createFrameFeed(preview.url, {
      onDecoded: options.onChanged,
      onFailed: (error) => report(key, error),
    });
    feedByKey.set(key, feed);
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
    feedKey,
    resolve(asset: VideoAsset, sourceMs: number, playing: boolean, slotKey?: string) {
      if (asset.kind === "image") return bitmapOf(asset);
      if (asset.kind !== "video") return undefined;
      return feedOf(asset, slotKey)?.frameAt(sourceMs / 1000, playing);
    },
    prewarmUpcoming(project: VideoEditProject, atMs: number, activeKeys: ReadonlySet<string>) {
      const assetById = new Map(project.assets.map((asset) => [asset.nodeId, asset]));
      for (const clip of project.videoTrack.clips) {
        const delta = clip.timelineStartMs - atMs;
        if (delta <= 0 || delta > 2_000) continue;
        const asset = assetById.get(clip.assetId);
        if (!asset || asset.kind !== "video") continue;
        const key = feedKey(asset, clip.id);
        if (!activeKeys.has(key)) feedOf(asset, clip.id)?.prewarm(clip.sourceInMs / 1000);
      }
    },
    syncActive(activeKeys: ReadonlySet<string>) {
      for (const [key, feed] of feedByKey) {
        if (!activeKeys.has(key)) feed.pause();
      }
    },
    pauseAll() {
      for (const feed of feedByKey.values()) feed.pause();
    },
    forget(asset: VideoAsset) {
      const assetKey = assetIdentity(asset);
      const prefix = `${assetKey}::`;
      for (const [key, feed] of feedByKey.entries()) {
        if (key === assetKey || key.startsWith(prefix)) {
          feed.dispose();
          feedByKey.delete(key);
        }
      }
      for (const key of failed) {
        if (key === assetKey || key.startsWith(prefix)) failed.delete(key);
      }
      options.videos.forget(asset);
      options.images.forget(asset);
    },
    /** Media that failed to open or decode is opened afresh on its next frame. */
    retry() {
      for (const key of failed) {
        feedByKey.get(key)?.dispose();
        feedByKey.delete(key);
      }
      failed.clear();
    },
    dispose() {
      for (const feed of feedByKey.values()) feed.dispose();
      feedByKey.clear();
      failed.clear();
    },
  };
}
