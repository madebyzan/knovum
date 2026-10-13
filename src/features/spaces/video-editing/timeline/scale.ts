const MIN_PX_PER_SECOND = 2;
export const MAX_PX_PER_SECOND = 480;
/** Trailing room so the last clip can be dragged and trimmed without fighting the viewport edge. */
const TRAILING_MS = 2_000;

const TICK_STEPS_MS = [
  100, 200, 500, 1_000, 2_000, 5_000, 10_000, 15_000, 30_000, 60_000, 120_000, 300_000, 600_000, 1_800_000,
];

export function clampPxPerSecond(value: number) {
  return Math.min(MAX_PX_PER_SECOND, Math.max(MIN_PX_PER_SECOND, value));
}

/** Scale that shows the whole project inside the viewport. */
export function fitPxPerSecond(durationMs: number, viewportWidth: number) {
  if (durationMs <= 0 || viewportWidth <= 0) return 60;
  return clampPxPerSecond((viewportWidth / (durationMs + TRAILING_MS)) * 1000);
}

export function timelineWidth(durationMs: number, pxPerSecond: number) {
  return ((durationMs + TRAILING_MS) * pxPerSecond) / 1000;
}

export function timeToPx(timeMs: number, pxPerSecond: number) {
  return (timeMs * pxPerSecond) / 1000;
}

export function pxToTime(px: number, pxPerSecond: number) {
  return (px * 1000) / pxPerSecond;
}

/** Smallest labelled step that keeps ruler labels at least 72 px apart. */
export function tickStepMs(pxPerSecond: number) {
  return TICK_STEPS_MS.find((step) => timeToPx(step, pxPerSecond) >= 72) ?? TICK_STEPS_MS.at(-1)!;
}

export function visibleTicks(options: {
  pxPerSecond: number;
  scrollLeft: number;
  viewportWidth: number;
  durationMs: number;
}) {
  const step = tickStepMs(options.pxPerSecond);
  const from = Math.max(0, Math.floor(pxToTime(options.scrollLeft, options.pxPerSecond) / step) * step);
  const to = Math.min(
    options.durationMs + TRAILING_MS,
    pxToTime(options.scrollLeft + options.viewportWidth, options.pxPerSecond) + step,
  );
  const ticks: number[] = [];
  for (let time = from; time <= to; time += step) ticks.push(Math.round(time));
  return ticks;
}

/** Keeps the time under the pointer fixed while zooming, the way editors and maps behave. */
export function zoomAroundPointer(options: {
  pxPerSecond: number;
  factor: number;
  scrollLeft: number;
  pointerX: number;
}) {
  const next = clampPxPerSecond(options.pxPerSecond * options.factor);
  const anchorMs = pxToTime(options.scrollLeft + options.pointerX, options.pxPerSecond);
  return {
    pxPerSecond: next,
    scrollLeft: Math.max(0, timeToPx(anchorMs, next) - options.pointerX),
  };
}

/** Scroll position that reveals a time, used to follow the playhead during playback. */
export function scrollToReveal(options: {
  timeMs: number;
  pxPerSecond: number;
  scrollLeft: number;
  viewportWidth: number;
}) {
  const x = timeToPx(options.timeMs, options.pxPerSecond);
  const leading = options.scrollLeft + options.viewportWidth * 0.1;
  const trailing = options.scrollLeft + options.viewportWidth * 0.85;
  if (x > trailing) return Math.max(0, x - options.viewportWidth * 0.5);
  if (x < leading) return Math.max(0, x - options.viewportWidth * 0.1);
  return options.scrollLeft;
}
