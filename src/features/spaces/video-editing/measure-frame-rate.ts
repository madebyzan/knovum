import { versionedMediaSource } from "../media/media-element-source";
import type { VideoMaterialNode } from "./material";
import type { VideoOutput } from "./project";

/** Frame rates the output contract supports; a source is matched to the closest one. */
const SUPPORTED: VideoOutput["frameRate"][] = [24, 25, 30, 60];
const SAMPLES = 5;
const TIMEOUT_MS = 2_000;

interface FrameSink {
  requestVideoFrameCallback?: (callback: (now: number, metadata: { mediaTime: number }) => void) => number;
}

/**
 * Reads the frame rate of a source by watching the frames it actually presents. The project output
 * then starts at the cadence of the video instead of a house default that silently resamples it.
 */
export async function measureFrameRate(
  node: VideoMaterialNode,
  signal: AbortSignal,
): Promise<VideoOutput["frameRate"] | undefined> {
  const element = document.createElement("video");
  element.muted = true;
  element.preload = "auto";
  element.playsInline = true;
  element.src = versionedMediaSource(node.data.assetPath, node.data.sourceRevision)!;
  try {
    const intervals = await sample(element, signal);
    return intervals.length > 0 ? classify(median(intervals)) : undefined;
  } catch {
    return undefined;
  } finally {
    element.pause();
    element.removeAttribute("src");
    element.load();
  }
}

function sample(element: HTMLVideoElement, signal: AbortSignal) {
  const sink = element as HTMLVideoElement & FrameSink;
  if (!sink.requestVideoFrameCallback) return Promise.resolve([]);
  return new Promise<number[]>((resolve, reject) => {
    const intervals: number[] = [];
    let previous: number | undefined;
    const timer = globalThis.setTimeout(() => finish(), TIMEOUT_MS);
    const abort = () => reject(signal.reason);
    const finish = () => {
      globalThis.clearTimeout(timer);
      signal.removeEventListener("abort", abort);
      resolve(intervals);
    };
    const onFrame = (_now: number, metadata: { mediaTime: number }) => {
      if (previous !== undefined && metadata.mediaTime > previous)
        intervals.push((metadata.mediaTime - previous) * 1000);
      previous = metadata.mediaTime;
      if (intervals.length >= SAMPLES) finish();
      else sink.requestVideoFrameCallback?.(onFrame);
    };
    signal.addEventListener("abort", abort, { once: true });
    sink.requestVideoFrameCallback(onFrame);
    element.play().catch(() => finish());
  });
}

function median(values: number[]) {
  const ordered = values.toSorted((left, right) => left - right);
  return ordered[Math.floor(ordered.length / 2)]!;
}

function classify(intervalMs: number) {
  const measured = 1000 / Math.max(1, intervalMs);
  return SUPPORTED.reduce((closest, candidate) =>
    Math.abs(candidate - measured) < Math.abs(closest - measured) ? candidate : closest,
  );
}
