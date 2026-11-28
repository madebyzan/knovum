import { computed, ref } from "vue";

import { formatDurationMs } from "./format-duration";

export function createRecordingTimer() {
  const elapsedMs = ref(0);
  const elapsedLabel = computed(() => formatDurationMs(elapsedMs.value));
  let timer: ReturnType<typeof globalThis.setInterval> | undefined;
  let accumulatedMs = 0;
  let segmentStartedAt: number | undefined;

  function start() {
    reset();
    segmentStartedAt = performance.now();
    timer = globalThis.setInterval(() => {
      if (segmentStartedAt !== undefined) {
        elapsedMs.value = accumulatedMs + (performance.now() - segmentStartedAt);
      }
    }, 200);
  }

  function pause() {
    if (segmentStartedAt === undefined) return;
    accumulatedMs += performance.now() - segmentStartedAt;
    segmentStartedAt = undefined;
    elapsedMs.value = accumulatedMs;
  }

  function resume() {
    if (segmentStartedAt !== undefined) return;
    segmentStartedAt = performance.now();
  }

  function reset() {
    globalThis.clearInterval(timer);
    timer = undefined;
    accumulatedMs = 0;
    segmentStartedAt = undefined;
    elapsedMs.value = 0;
  }

  return { elapsedMs, elapsedLabel, start, pause, resume, reset };
}
