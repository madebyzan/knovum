import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import { prepareVideoEditProxy } from "../prepare-video-edit-proxy";
import type { VideoAsset } from "../project";

export interface PreviewSource {
  kind: VideoAsset["kind"];
  url: string;
}

/**
 * Resolves one playable URL per asset and keeps it: video decodes from the short-GOP editing proxy
 * Rust prepares, while images and audio play their managed file directly. Preparation runs once per
 * asset revision instead of once per view that happens to show the clip.
 */
export function createPreviewSources(spaceId: string, signal: AbortSignal) {
  const resolved = new Map<string, PreviewSource>();
  const pending = new Map<string, Promise<PreviewSource>>();

  function ready(asset: VideoAsset) {
    return resolved.get(identity(asset));
  }

  function load(asset: VideoAsset, node: VideoMaterialNode): Promise<PreviewSource> {
    const key = identity(asset);
    const known = resolved.get(key);
    if (known) return Promise.resolve(known);
    const inFlight = pending.get(key);
    if (inFlight) return inFlight;
    const request = resolve(asset, node)
      .then((value) => {
        resolved.set(key, value);
        return value;
      })
      .finally(() => pending.delete(key));
    pending.set(key, request);
    return request;
  }

  async function resolve(asset: VideoAsset, node: VideoMaterialNode) {
    const path =
      asset.kind === "video"
        ? await prepareVideoEditProxy(
            { spaceId, sourceNodeId: asset.nodeId, sourceRevision: asset.sourceRevision },
            signal,
          )
        : node.data.assetPath;
    return { kind: asset.kind, url: versionedMediaSource(path, asset.sourceRevision)! };
  }

  return { ready, load, forget: (asset: VideoAsset) => resolved.delete(identity(asset)) };
}

export type PreviewSources = ReturnType<typeof createPreviewSources>;

function identity(asset: VideoAsset) {
  return `${asset.nodeId}\u0000${asset.sourceRevision}`;
}
