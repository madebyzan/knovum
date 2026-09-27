import { clamp } from "es-toolkit";
import { shallowRef, watch, type Ref } from "vue";

import { versionedMediaSource } from "../media/element";

export interface TimelineFrame {
  startMs: number;
  endMs: number;
  image: string;
}

const MAX_FRAMES = 24;
const frameCache = new Map<string, TimelineFrame[]>();

export function useVideoTimelineThumbnails(sourcePath: Ref<string | undefined>, durationMs: Ref<number>) {
  const frames = shallowRef<TimelineFrame[]>([]);
  const error = shallowRef(false);

  watch(
    [sourcePath, durationMs],
    ([path, duration], _previous, onCleanup) => {
      frames.value = [];
      error.value = false;
      if (!path || duration <= 0) return;
      const cacheKey = `${path}\0${duration}`;
      const cached = frameCache.get(cacheKey);
      if (cached) {
        frames.value = cached;
        return;
      }

      const abort = new AbortController();
      const video = document.createElement("video");
      onCleanup(() => {
        abort.abort();
        video.removeAttribute("src");
        video.load();
      });
      video.crossOrigin = "anonymous";
      video.muted = true;
      video.preload = "auto";
      void loadFrames({ video, path, duration, cacheKey, abort, frames, error });
    },
    { immediate: true },
  );

  return { frames, error };
}

interface FrameLoad {
  video: HTMLVideoElement;
  path: string;
  duration: number;
  cacheKey: string;
  abort: AbortController;
  frames: Ref<TimelineFrame[]>;
  error: Ref<boolean>;
}

async function loadFrames(load: FrameLoad) {
  const { video, path, duration, cacheKey, abort, frames, error } = load;
  try {
    await waitForVideo(video, "loadedmetadata", abort.signal, () => {
      video.src = versionedMediaSource(path)!;
      video.load();
    });
    const mediaDuration = Math.min(duration, Math.floor(video.duration * 1_000));
    if (!Number.isFinite(mediaDuration) || mediaDuration <= 0) return;
    const canvas = document.createElement("canvas");
    canvas.width = 96;
    canvas.height = 54;
    const context = canvas.getContext("2d");
    if (!context) return;
    const count = clamp(Math.ceil(mediaDuration / 2_500), 1, MAX_FRAMES);
    await sampleFrames(load, { canvas, context, duration: mediaDuration, count }, 0);
    if (!abort.signal.aborted) {
      if (frameCache.size >= 8) frameCache.delete(frameCache.keys().next().value!);
      frameCache.set(cacheKey, frames.value);
    }
  } catch {
    if (!abort.signal.aborted) {
      frames.value = [];
      error.value = true;
    }
  } finally {
    video.removeAttribute("src");
    video.load();
  }
}

interface FrameSampler {
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
  duration: number;
  count: number;
}

async function sampleFrames(load: FrameLoad, sampler: FrameSampler, index: number): Promise<void> {
  if (index >= sampler.count || load.abort.signal.aborted) return;
  const { canvas, context, duration, count } = sampler;
  const startMs = (index / count) * duration;
  const endMs = ((index + 1) / count) * duration;
  const sourceMs = Math.min(duration - 1, (startMs + endMs) / 2);
  await waitForVideo(load.video, "seeked", load.abort.signal, () => {
    load.video.currentTime = sourceMs / 1_000;
  });
  context.drawImage(load.video, 0, 0, canvas.width, canvas.height);
  load.frames.value = [...load.frames.value, { startMs, endMs, image: canvas.toDataURL("image/jpeg", 0.62) }];
  await sampleFrames(load, sampler, index + 1);
}

function waitForVideo(
  video: HTMLVideoElement,
  eventName: "loadedmetadata" | "seeked",
  signal: AbortSignal,
  start: () => void,
) {
  return new Promise<void>((resolve, reject) => {
    const finish = (error?: Error) => {
      video.removeEventListener(eventName, onReady);
      video.removeEventListener("error", onError);
      signal.removeEventListener("abort", onAbort);
      if (error) reject(error);
      else resolve();
    };
    const onReady = () => finish();
    const onError = () => finish(new Error("Video thumbnail decoding failed"));
    const onAbort = () => finish(new Error("Video thumbnail decoding cancelled"));
    video.addEventListener(eventName, onReady, { once: true });
    video.addEventListener("error", onError, { once: true });
    signal.addEventListener("abort", onAbort, { once: true });
    start();
  });
}
