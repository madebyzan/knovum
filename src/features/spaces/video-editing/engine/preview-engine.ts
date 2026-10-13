import type { VideoMaterialNode } from "../material";
import type { VideoAsset, VideoEditProject, VideoTextOverlay } from "../project";
import { lastFrameMs, projectDurationMs } from "../timing";
import { createAudioGraph } from "./audio-graph";
import { createCompositor, type CompositionLayer, type Compositor } from "./compositor";
import { planPreview, type PreviewPlan } from "./layer-plan";
import { createMediaPool } from "./media-pool";
import { createPlaybackClock } from "./playback-clock";
import { createPreviewSources } from "./preview-sources";

interface PreviewEngineOptions {
  spaceId: string;
  project: () => VideoEditProject | undefined;
  materials: () => ReadonlyMap<string, VideoMaterialNode>;
  onFailed: (error: unknown) => void;
}

/**
 * Drives the preview: one clock, one pooled element per asset, one canvas frame per animation
 * frame. Media is corrected towards the clock and pre-rolled before a cut, so playback runs
 * continuously instead of re-seeking itself into a stutter.
 */
export function createPreviewEngine(options: PreviewEngineOptions) {
  const controller = new AbortController();
  const sources = createPreviewSources(options.spaceId, controller.signal);
  const audio = createAudioGraph();
  const pool = createMediaPool({ sources, audio, onFailed: options.onFailed });
  const clock = createPlaybackClock({
    durationMs: () => (options.project() ? projectDurationMs(options.project()!) : 0),
    lastFrameMs: () => (options.project() ? lastFrameMs(options.project()!) : 0),
  });
  let compositor: Compositor | undefined;
  let handle: number | undefined;
  let ready = true;
  let dirty = true;

  function invalidate() {
    dirty = true;
  }

  function attach(canvas: HTMLCanvasElement) {
    compositor = createCompositor(canvas);
    invalidate();
  }

  function layerFor(plan: PreviewPlan, retained: Set<string>) {
    const materials = options.materials();
    const drawable: CompositionLayer[] = [];
    let settled = true;
    for (const layer of plan.layers) {
      const node = materials.get(layer.asset.nodeId);
      if (!node) continue;
      const entry = pool.prepare(layer.asset, node, layer.sourceMs);
      retained.add(pool.identity(layer.asset));
      if (!entry) {
        settled = false;
        continue;
      }
      const synchronized = pool.sync(entry, {
        sourceMs: layer.sourceMs,
        playing: clock.playing.value,
        gain: layer.gain,
      });
      settled = settled && synchronized;
      if (entry.presentable)
        drawable.push({
          media: entry.element as CanvasImageSource,
          asset: layer.asset,
          transform: layer.clip.transform,
          fade: layer.fade,
        });
    }
    return { drawable, settled };
  }

  function audioFor(plan: PreviewPlan, retained: Set<string>) {
    const materials = options.materials();
    let settled = true;
    for (const track of plan.audio) {
      const node = materials.get(track.asset.nodeId);
      if (!node) continue;
      const entry = pool.prepare(track.asset, node, track.sourceMs);
      retained.add(pool.identity(track.asset));
      if (!entry) {
        settled = false;
        continue;
      }
      settled =
        pool.sync(entry, {
          sourceMs: track.sourceMs,
          playing: clock.playing.value,
          gain: track.gain,
        }) && settled;
    }
    return settled;
  }

  function preload(plan: PreviewPlan, retained: Set<string>) {
    const materials = options.materials();
    for (const item of plan.preroll) {
      const node = materials.get(item.asset.nodeId);
      if (!node) continue;
      pool.prepare(item.asset, node, item.sourceMs);
      retained.add(pool.identity(item.asset));
    }
  }

  function frame(now: number) {
    handle = requestAnimationFrame(frame);
    clock.tick(now, ready);
    const project = options.project();
    if (!project || !compositor) return;
    if (!clock.playing.value && !dirty && ready) return;
    const plan = planPreview(project, Math.min(clock.timeMs.value, lastFrameMs(project)));
    const retained = new Set<string>();
    const layers = layerFor(plan, retained);
    ready = audioFor(plan, retained) && layers.settled;
    preload(plan, retained);
    pool.retain(retained);
    compositor.draw({
      output: project.output,
      layers: layers.drawable,
      transition: layers.drawable.length === 2 ? plan.transition : undefined,
      texts: plan.texts,
    });
    dirty = !ready;
  }

  return {
    timeMs: clock.timeMs,
    playing: clock.playing,
    stalled: clock.stalled,
    sources,
    seek(value: number) {
      clock.seek(value);
      invalidate();
    },
    play: clock.play,
    pause() {
      clock.pause();
      pool.pauseAll();
      invalidate();
    },
    toggle: clock.toggle,
    attach,
    invalidate,
    measureText(item: VideoTextOverlay) {
      const project = options.project();
      return project ? compositor?.measureText(item, project.output) : undefined;
    },
    forget(asset: VideoAsset) {
      sources.forget(asset);
      invalidate();
    },
    start() {
      handle ??= requestAnimationFrame(frame);
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
      controller.abort();
      pool.dispose();
      audio.dispose();
      compositor = undefined;
    },
  };
}
