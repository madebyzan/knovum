import { shallowRef } from "vue";

import type { VideoMaterialNode } from "../material";
import type { VideoAsset, VideoEditProject, VideoTextOverlay } from "../project";
import { lastFrameMs, projectDurationMs } from "../timing";
import { createAudioGraph } from "./audio-graph";
import { createCompositor, type Compositor } from "./compositor";
import { createFrameResolver } from "./frame-resolver";
import { planPreview } from "./layer-plan";
import { createMediaPool } from "./media-pool";
import { createPlaybackClock } from "./playback-clock";
import { createPreviewSources } from "./preview-sources";

interface PreviewEngineOptions {
  spaceId: string;
  project: () => VideoEditProject | undefined;
  materials: () => ReadonlyMap<string, VideoMaterialNode>;
  onFailed: (error: unknown) => void;
}

/** Cadence of the timecode shown as text; the picture itself follows the animation frame. */
const DISPLAY_INTERVAL_MS = 66;

type FrameListener = (timeMs: number, playing: boolean) => void;

/**
 * Drives the preview: one clock, one pooled element per asset, one canvas frame per animation
 * frame. A frame is painted only when every layer it needs can be shown, so a slow decode holds
 * the last picture instead of flashing the background, and the clock waits with it.
 */
export function createPreviewEngine(options: PreviewEngineOptions) {
  const controller = new AbortController();
  const sources = createPreviewSources(options.spaceId, controller.signal);
  const audio = createAudioGraph();
  const pool = createMediaPool({
    sources,
    audio,
    onChanged: invalidate,
    onFailed: options.onFailed,
  });
  const clock = createPlaybackClock({
    durationMs: () => measured()?.duration ?? 0,
    lastFrameMs: () => measured()?.last ?? 0,
  });
  const resolver = createFrameResolver({
    pool,
    materials: options.materials,
    playing: () => clock.playing.value,
  });
  const displayMs = shallowRef(0);
  const listeners = new Set<FrameListener>();
  let compositor: Compositor | undefined;
  let handle: number | undefined;
  let ready = true;
  let dirty = true;
  let publishedAt = 0;

  function measured() {
    const value = options.project();
    if (!value) return undefined;
    return { value, duration: projectDurationMs(value), last: lastFrameMs(value) };
  }

  function invalidate() {
    dirty = true;
  }

  function publish(force: boolean) {
    for (const listener of listeners) listener(clock.timeMs.value, clock.playing.value);
    const now = performance.now();
    if (!force && now - publishedAt < DISPLAY_INTERVAL_MS) return;
    publishedAt = now;
    displayMs.value = clock.timeMs.value;
  }

  function render(project: VideoEditProject, position: number) {
    const plan = planPreview(project, position);
    const frame = resolver.resolve(plan);
    pool.retain(frame.retained);
    ready = frame.complete;
    dirty = false;
    if (!ready || !compositor) return;
    compositor.draw({
      output: project.output,
      layers: frame.layers,
      transition: frame.layers.length === 2 ? plan.transition : undefined,
      texts: plan.texts,
    });
  }

  function tick(now: number) {
    handle = requestAnimationFrame(tick);
    const current = measured();
    if (!current || !compositor) return;
    if (clock.playing.value) {
      clock.tick(now, ready);
      publish(false);
    } else if (!dirty) return;
    render(current.value, Math.min(clock.timeMs.value, current.last));
  }

  function halt() {
    clock.pause();
    pool.pauseAll();
    invalidate();
  }

  return {
    playing: clock.playing,
    stalled: clock.stalled,
    displayMs,
    sources,
    audio,
    positionMs: () => clock.timeMs.value,
    subscribe(listener: FrameListener) {
      listeners.add(listener);
      listener(clock.timeMs.value, clock.playing.value);
      return () => listeners.delete(listener);
    },
    seek(value: number) {
      clock.seek(value);
      invalidate();
      publish(true);
    },
    play() {
      clock.play();
      publish(true);
    },
    pause() {
      halt();
      publish(true);
    },
    toggle() {
      if (clock.playing.value) halt();
      else clock.play();
      publish(true);
    },
    attach(canvas: HTMLCanvasElement) {
      compositor = createCompositor(canvas);
      invalidate();
    },
    invalidate,
    measureText(item: VideoTextOverlay) {
      const current = options.project();
      return current ? compositor?.measureText(item, current.output) : undefined;
    },
    forget(asset: VideoAsset) {
      sources.forget(asset);
      invalidate();
    },
    start() {
      handle ??= requestAnimationFrame(tick);
    },
    stop() {
      if (handle !== undefined) cancelAnimationFrame(handle);
      handle = undefined;
      clock.pause();
      pool.pauseAll();
    },
    dispose() {
      if (handle !== undefined) cancelAnimationFrame(handle);
      handle = undefined;
      listeners.clear();
      controller.abort();
      pool.dispose();
      audio.dispose();
      compositor = undefined;
    },
  };
}
