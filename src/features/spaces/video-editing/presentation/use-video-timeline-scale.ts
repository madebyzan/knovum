import { useResizeObserver } from "@vueuse/core";
import { computed, nextTick, shallowRef, watch, type Ref } from "vue";

import { type VideoEditProject } from "../project";
import { projectDurationMs } from "../timing";

const TICK_INTERVALS_MS = [
  100, 250, 500, 1_000, 2_000, 5_000, 10_000, 15_000, 30_000, 60_000, 120_000, 300_000, 600_000,
];

interface VideoTimelineScaleInput {
  project: Ref<VideoEditProject>;
  zoom: () => number;
  playheadMs: () => number;
  playing: () => boolean;
}

export function useVideoTimelineScale({ project, zoom, playheadMs, playing }: VideoTimelineScaleInput) {
  const viewport = shallowRef<HTMLElement | null>(null);
  const viewportWidth = shallowRef(0);
  const durationMs = computed(() => Math.max(projectDurationMs(project.value), 1));
  const trackWidth = computed(() => Math.max(viewportWidth.value, 1) * zoom());
  const contentStyle = computed(() => ({ width: `${trackWidth.value}px` }));
  const ticks = computed(() => {
    const pixelsPerMs = trackWidth.value / durationMs.value;
    const interval =
      TICK_INTERVALS_MS.find((value) => value * pixelsPerMs >= 80) ?? TICK_INTERVALS_MS.at(-1)!;
    return Array.from(
      { length: Math.floor(durationMs.value / interval) + 1 },
      (_, index) => index * interval,
    );
  });
  const clipStyle = (milliseconds: number) => ({ width: `${(milliseconds / durationMs.value) * 100}%` });

  useResizeObserver(viewport, ([entry]) => {
    viewportWidth.value = entry?.contentRect.width ?? 0;
  });

  watch(zoom, async () => {
    await nextTick();
    if (viewport.value)
      viewport.value.scrollLeft =
        (playheadMs() / durationMs.value) * trackWidth.value - viewportWidth.value / 2;
  });
  watch(playheadMs, () => {
    const element = viewport.value;
    if (!element || !playing() || zoom() === 1) return;
    const x = (playheadMs() / durationMs.value) * trackWidth.value;
    if (x > element.scrollLeft + viewportWidth.value * 0.85)
      element.scrollLeft = x - viewportWidth.value * 0.35;
  });

  return { viewport, durationMs, ticks, contentStyle, clipStyle };
}
