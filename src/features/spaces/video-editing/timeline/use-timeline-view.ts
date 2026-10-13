import { useEventListener, useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, watch } from "vue";

import type { VideoEditProject } from "../project";
import { projectDurationMs } from "../timing";
import {
  clampPxPerSecond,
  fitPxPerSecond,
  pxToTime,
  scrollToReveal,
  timelineWidth,
  visibleTicks,
  zoomAroundPointer,
} from "./scale";

interface TimelineViewOptions {
  project: () => VideoEditProject | undefined;
  playheadMs: () => number;
  playing: () => boolean;
}

/**
 * Owns the timeline viewport: a pixels-per-second scale, the scroll window, ruler ticks, zooming
 * around the pointer and following the playhead during playback.
 */
export function useTimelineView(options: TimelineViewOptions) {
  const viewport = shallowRef<HTMLElement | null>(null);
  const surface = shallowRef<HTMLElement | null>(null);
  const viewportWidth = shallowRef(0);
  const scrollLeft = shallowRef(0);
  const pxPerSecond = shallowRef(60);
  let userScaled = false;

  const durationMs = computed(() => {
    const project = options.project();
    return project ? projectDurationMs(project) : 0;
  });
  const contentWidth = computed(() =>
    Math.max(viewportWidth.value, timelineWidth(durationMs.value, pxPerSecond.value)),
  );
  const ticks = computed(() =>
    visibleTicks({
      pxPerSecond: pxPerSecond.value,
      scrollLeft: scrollLeft.value,
      viewportWidth: viewportWidth.value,
      durationMs: durationMs.value,
    }),
  );

  useResizeObserver(viewport, ([entry]) => {
    viewportWidth.value = entry?.contentRect.width ?? 0;
    if (!userScaled) fit();
  });
  useEventListener(viewport, "scroll", () => {
    scrollLeft.value = viewport.value?.scrollLeft ?? 0;
  });

  function fit() {
    pxPerSecond.value = fitPxPerSecond(durationMs.value, viewportWidth.value);
    userScaled = false;
    if (viewport.value) viewport.value.scrollLeft = 0;
  }

  function zoom(factor: number, pointerX = viewportWidth.value / 2) {
    const next = zoomAroundPointer({
      pxPerSecond: pxPerSecond.value,
      factor,
      scrollLeft: scrollLeft.value,
      pointerX,
    });
    pxPerSecond.value = next.pxPerSecond;
    userScaled = true;
    if (viewport.value) viewport.value.scrollLeft = next.scrollLeft;
  }

  function setPxPerSecond(value: number) {
    pxPerSecond.value = clampPxPerSecond(value);
    userScaled = true;
  }

  /** Ctrl/Cmd plus wheel zooms around the pointer; a plain wheel scrolls the lanes. */
  function onWheel(event: WheelEvent) {
    if (!(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    const bounds = viewport.value?.getBoundingClientRect();
    zoom(event.deltaY < 0 ? 1.15 : 1 / 1.15, bounds ? event.clientX - bounds.left : undefined);
  }

  watch([() => options.playheadMs(), () => options.playing()], () => {
    const element = viewport.value;
    if (!element || !options.playing()) return;
    element.scrollLeft = scrollToReveal({
      timeMs: options.playheadMs(),
      pxPerSecond: pxPerSecond.value,
      scrollLeft: element.scrollLeft,
      viewportWidth: viewportWidth.value,
    });
  });

  function timeAt(event: { clientX: number }) {
    const element = surface.value;
    if (!element) return 0;
    return Math.max(0, pxToTime(event.clientX - element.getBoundingClientRect().left, pxPerSecond.value));
  }

  return {
    viewport,
    surface,
    timeAt,
    viewportWidth,
    scrollLeft,
    pxPerSecond,
    contentWidth,
    durationMs,
    ticks,
    fit,
    zoom,
    setPxPerSecond,
    onWheel,
    reveal(timeMs: number) {
      const element = viewport.value;
      if (!element) return;
      element.scrollLeft = scrollToReveal({
        timeMs,
        pxPerSecond: pxPerSecond.value,
        scrollLeft: element.scrollLeft,
        viewportWidth: viewportWidth.value,
      });
    },
  };
}
