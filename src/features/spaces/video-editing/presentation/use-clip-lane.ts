import { useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, type ShallowRef } from "vue";

import type { VideoAudioClip, VideoClip } from "../project";
import { timeToPx } from "../timeline/scale";
import type { useTimelineView } from "../timeline/use-timeline-view";
import { clipLengthMs } from "../timing";

/** Device pixels per CSS pixel a lane is drawn at; more costs memory without a visible gain. */
const MAX_PIXEL_RATIO = 2;

interface LaneSurface {
  context: CanvasRenderingContext2D;
  /** CSS pixels the context draws in. */
  width: number;
  height: number;
  /** The source time the surface spans, left edge to right edge. */
  source: { fromMs: number; toMs: number };
}

/**
 * The canvas a clip lane draws its thumbnails or waveform on. It covers only the part of the clip
 * inside the timeline's visible window: a long source at a close zoom is far wider than any canvas
 * a WebView can allocate, and nothing outside the window would be seen anyway.
 */
export function useClipLane(
  clip: () => VideoClip | VideoAudioClip,
  timeline: Pick<ReturnType<typeof useTimelineView>, "visible" | "pxPerSecond">,
  canvas: Readonly<ShallowRef<HTMLCanvasElement | null>>,
) {
  const height = shallowRef(0);
  useResizeObserver(canvas, ([entry]) => {
    height.value = entry?.contentRect.height ?? 0;
  });

  const slice = computed(() => {
    const value = clip();
    const startMs = value.timelineStartMs;
    const fromMs = Math.max(startMs, timeline.visible.value.fromMs);
    const toMs = Math.min(startMs + clipLengthMs(value), timeline.visible.value.toMs);
    if (toMs <= fromMs) return undefined;
    const pxPerSecond = timeline.pxPerSecond.value;
    const shift = value.sourceInMs - startMs;
    return {
      leftPx: timeToPx(fromMs - startMs, pxPerSecond),
      widthPx: Math.max(1, Math.round(timeToPx(toMs - fromMs, pxPerSecond))),
      source: { fromMs: fromMs + shift, toMs: toMs + shift },
    };
  });

  return {
    style: computed(() =>
      slice.value
        ? { left: `${slice.value.leftPx}px`, width: `${slice.value.widthPx}px` }
        : { display: "none" },
    ),
    /** Sizes the canvas for the visible slice and clears it; nothing when nothing is visible. */
    surface(): LaneSurface | undefined {
      const element = canvas.value;
      const value = slice.value;
      const cssHeight = Math.round(height.value);
      if (!element || !value || cssHeight <= 0) return undefined;
      const ratio = Math.min(globalThis.devicePixelRatio || 1, MAX_PIXEL_RATIO);
      element.width = value.widthPx * ratio;
      element.height = cssHeight * ratio;
      const context = element.getContext("2d");
      if (!context) return undefined;
      context.setTransform(ratio, 0, 0, ratio, 0, 0);
      context.clearRect(0, 0, value.widthPx, cssHeight);
      return { context, width: value.widthPx, height: cssHeight, source: value.source };
    },
  };
}
