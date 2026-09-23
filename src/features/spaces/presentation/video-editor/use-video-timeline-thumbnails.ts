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
    async ([path, duration], _previous, onCleanup) => {
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

        const count = Math.min(MAX_FRAMES, Math.max(1, Math.ceil(mediaDuration / 2_500)));
        for (let index = 0; index < count && !abort.signal.aborted; index++) {
          const startMs = (index / count) * mediaDuration;
          const endMs = ((index + 1) / count) * mediaDuration;
          const sourceMs = Math.min(mediaDuration - 1, (startMs + endMs) / 2);
          await waitForVideo(video, "seeked", abort.signal, () => {
            video.currentTime = sourceMs / 1_000;
          });
          context.drawImage(video, 0, 0, canvas.width, canvas.height);
          frames.value = [...frames.value, { startMs, endMs, image: canvas.toDataURL("image/jpeg", 0.62) }];
        }
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
    },
    { immediate: true },
  );

  return { frames, error };
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
