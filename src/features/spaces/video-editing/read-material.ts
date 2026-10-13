import { ALL_FORMATS, Input, UrlSource } from "mediabunny";

import { versionedMediaSource } from "../media/media-element-source";
import { measuredVideoAsset, type VideoMaterialNode } from "./material";
import type { VideoAsset, VideoOutput } from "./project";

/** Frame rates the output contract supports; a source is matched to the closest one. */
const SUPPORTED: VideoOutput["frameRate"][] = [24, 25, 30, 60];
/** Packets sampled to estimate cadence; enough to tell the supported rates apart. */
const STATS_SAMPLE = 100;

export interface MeasuredMaterial {
  asset: VideoAsset;
  /** Measured source cadence, when the file states one. */
  frameRate: VideoOutput["frameRate"] | undefined;
}

function classify(rate: number) {
  if (!Number.isFinite(rate) || rate <= 0) return undefined;
  return SUPPORTED.reduce((closest, candidate) =>
    Math.abs(candidate - rate) < Math.abs(closest - rate) ? candidate : closest,
  );
}

async function measureImage(node: VideoMaterialNode, url: string, signal: AbortSignal) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw new Error("Cannot read the selected image");
  const bitmap = await createImageBitmap(await response.blob());
  try {
    return {
      asset: measuredVideoAsset(node, { durationMs: 0, width: bitmap.width, height: bitmap.height }),
      frameRate: undefined,
    };
  } finally {
    bitmap.close();
  }
}

/**
 * Measures a material from its original file. Mediabunny parses the container, so admitting media
 * costs no transcode and no playback: the measurements describe the source itself, including the
 * cadence the project output starts at, rather than a downscaled or resampled copy of it.
 */
export async function readVideoMaterial(
  node: VideoMaterialNode,
  signal: AbortSignal,
): Promise<MeasuredMaterial> {
  if (signal.aborted) throw signal.reason;
  const url = versionedMediaSource(node.data.assetPath, node.data.sourceRevision)!;
  if (node.data.kind === "image") return measureImage(node, url, signal);
  const input = new Input({ formats: ALL_FORMATS, source: new UrlSource(url) });
  const abort = () => input.dispose();
  signal.addEventListener("abort", abort, { once: true });
  try {
    const video = await input.getPrimaryVideoTrack();
    const durationMs = Math.round((await input.computeDuration()) * 1000);
    const stats = video ? await video.computePacketStats(STATS_SAMPLE) : undefined;
    return {
      asset: measuredVideoAsset(node, {
        durationMs,
        width: video?.displayWidth ?? 0,
        height: video?.displayHeight ?? 0,
      }),
      frameRate: stats ? classify(stats.averagePacketRate) : undefined,
    };
  } finally {
    signal.removeEventListener("abort", abort);
    input.dispose();
  }
}
