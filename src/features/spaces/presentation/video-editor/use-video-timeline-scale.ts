import { useResizeObserver } from "@vueuse/core";
import { computed, nextTick, shallowRef, watch, type Ref } from "vue";

import { projectDuration, type VideoEditProject } from "../../video-editing/project";

const TICK_INTERVALS = [
  100, 250, 500, 1_000, 2_000, 5_000, 10_000, 15_000, 30_000, 60_000, 120_000, 300_000, 600_000,
];

interface VideoTimelineScaleInput {
  project: Ref<VideoEditProject>;
  zoom: () => number;
  playhead: () => number;
  playing: () => boolean;
}

export function useVideoTimelineScale({ project, zoom, playhead, playing }: VideoTimelineScaleInput) {
  const viewport = shallowRef<HTMLElement | null>(null);
  const viewportWidth = shallowRef(0);
  const duration = computed(() => Math.max(projectDuration(project.value), 1));
  const trackWidth = computed(() => Math.max(viewportWidth.value, 1) * zoom());
  const contentStyle = computed(() => ({ width: `${trackWidth.value}px` }));
  const ticks = computed(() => {
    const pixelsPerMs = trackWidth.value / duration.value;
    const interval = TICK_INTERVALS.find((value) => value * pixelsPerMs >= 80) ?? TICK_INTERVALS.at(-1)!;
    return Array.from({ length: Math.floor(duration.value / interval) + 1 }, (_, index) => index * interval);
  });
  const clipStyle = (milliseconds: number) => ({ width: `${(milliseconds / duration.value) * 100}%` });

  useResizeObserver(viewport, ([entry]) => {
    viewportWidth.value = entry?.contentRect.width ?? 0;
  });

  watch(zoom, async () => {
    await nextTick();
    if (viewport.value)
      viewport.value.scrollLeft = (playhead() / duration.value) * trackWidth.value - viewportWidth.value / 2;
  });
  watch(playhead, () => {
    const element = viewport.value;
    if (!element || !playing() || zoom() === 1) return;
    const x = (playhead() / duration.value) * trackWidth.value;
    if (x > element.scrollLeft + viewportWidth.value * 0.85)
      element.scrollLeft = x - viewportWidth.value * 0.35;
  });

  return { viewport, duration, ticks, contentStyle, clipStyle };
}
