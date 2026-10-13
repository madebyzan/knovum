import { versionedMediaSource } from "../media/media-element-source";
import { measuredVideoAsset, type VideoMaterialNode } from "./material";

/**
 * Measures a material from its original file. Metadata parsing needs no transcode, so admitting
 * media never waits for an editing proxy, and the measurements describe the source itself rather
 * than a downscaled copy of it.
 */
export async function readVideoMaterial(node: VideoMaterialNode, signal: AbortSignal) {
  if (signal.aborted) throw signal.reason;
  const element = node.data.kind === "image" ? new Image() : document.createElement(node.data.kind);
  const eventName = node.data.kind === "image" ? "load" : "loadedmetadata";
  const url = versionedMediaSource(node.data.assetPath, node.data.sourceRevision)!;
  try {
    await new Promise<void>((resolve, reject) => {
      const finish = (error?: unknown) => {
        element.removeEventListener(eventName, ready);
        element.removeEventListener("error", failed);
        signal.removeEventListener("abort", abort);
        if (error) reject(error);
        else resolve();
      };
      const ready = () => finish();
      const failed = () => finish(new Error("Cannot read the selected media"));
      const abort = () => finish(signal.reason);
      element.addEventListener(eventName, ready, { once: true });
      element.addEventListener("error", failed, { once: true });
      signal.addEventListener("abort", abort, { once: true });
      element.src = url;
      if (element instanceof HTMLMediaElement) {
        element.preload = "metadata";
        element.load();
      }
    });
    if (element instanceof HTMLImageElement)
      return measuredVideoAsset(node, {
        durationMs: 0,
        width: element.naturalWidth,
        height: element.naturalHeight,
      });
    return measuredVideoAsset(node, {
      durationMs: Math.round(element.duration * 1000),
      width: element instanceof HTMLVideoElement ? element.videoWidth : 0,
      height: element instanceof HTMLVideoElement ? element.videoHeight : 0,
    });
  } finally {
    element.removeAttribute("src");
    if (element instanceof HTMLMediaElement) element.load();
  }
}
