import { shallowRef } from "vue";

import type { VideoMaterialNode } from "../material";
import { createAudioLibrary } from "../media/preview-audio";
import { createImageLibrary } from "../media/preview-images";
import { createVideoLibrary } from "../media/preview-video";
import type { VideoAsset, VideoEditProject } from "../project";
import { createStageScene, type StageScene } from "../stage/konva-scene";
import { lastFrameMs, projectDurationMs } from "../timing";
import { audioContext } from "./audio-context";
import { createFrameSources } from "./frame-sources";
import { createPlaybackClock } from "./playback-clock";
import { buildScene } from "./scene-plan";
import { createSoundTrack } from "./sound-track";

interface PlayerOptions {
  spaceId: string;
  project: () => VideoEditProject | undefined;
  materials: () => ReadonlyMap<string, VideoMaterialNode>;
  onFailed: (error: unknown) => void;
}

/** Cadence of the timecode shown as text; the picture itself follows the animation frame. */
const DISPLAY_INTERVAL_MS = 66;

type FrameListener = (timeMs: number, playing: boolean) => void;

/**
 * Composition root of the preview. The audio timeline is the clock, and every animation frame
 * presents the newest decoded picture for the clock's position. Nothing drives a media element, so
 * playback runs continuously and a slow decode repeats a frame instead of stalling the transport.
 */
export function createPreviewPlayer(options: PlayerOptions) {
  const controller = new AbortController();
  const videos = createVideoLibrary(options.spaceId, controller.signal);
  const images = createImageLibrary(options.materials);
  const audio = createAudioLibrary({ context: audioContext, videos, materials: options.materials });
  const sources = createFrameSources({ videos, images, onChanged: invalidate, onFailed: options.onFailed });
  const clock = createPlaybackClock({ context: audioContext, durationMs });
  const sound = createSoundTrack({ project: options.project, library: audio, clock, onFailed: options.onFailed });
  const displayMs = shallowRef(0);
  const listeners = new Set<FrameListener>();
  let scene: StageScene | undefined;
  let handle: number | undefined;
  let dirty = true;
  let publishedAt = 0;
  let viewScale = 1;

  function durationMs() {
    const project = options.project();
    return project ? projectDurationMs(project) : 0;
  }

  function invalidate() {
    dirty = true;
  }

  function publish(force: boolean) {
    const at = clock.positionMs();
    for (const listener of listeners) listener(at, clock.playing.value);
    const now = performance.now();
    if (!force && now - publishedAt < DISPLAY_INTERVAL_MS) return;
    publishedAt = now;
    displayMs.value = at;
  }

  function halt() {
    clock.pause();
    sound.stop();
    invalidate();
  }

  function render(project: VideoEditProject, playing: boolean) {
    if (!scene) return;
    scene.resize(project.output, viewScale);
    const at = Math.min(clock.positionMs(), lastFrameMs(project));
    scene.present(buildScene(project, at, (asset, sourceMs) => sources.resolve(asset, sourceMs, playing)));
  }

  function seek(value: number) {
    clock.seek(value);
    sound.reposition();
    invalidate();
    publish(true);
  }

  async function play() {
    if (clock.playing.value || !options.project()) return;
    await clock.play();
    sound.prepare();
    invalidate();
    publish(true);
  }

  function pause() {
    halt();
    publish(true);
  }

  function tick() {
    handle = requestAnimationFrame(tick);
    const project = options.project();
    if (!project || !scene) return;
    if (clock.endReached()) {
      halt();
      seek(lastFrameMs(project));
    }
    const playing = clock.playing.value;
    if (!playing && !dirty) return;
    dirty = false;
    render(project, playing);
    publish(!playing);
  }

  return {
    playing: clock.playing,
    displayMs,
    videos,
    audio,
    positionMs: () => clock.positionMs(),
    subscribe(listener: FrameListener) {
      listeners.add(listener);
      listener(clock.positionMs(), clock.playing.value);
      return () => listeners.delete(listener);
    },
    seek,
    play,
    pause,
    toggle() {
      if (clock.playing.value) return (pause(), Promise.resolve());
      return play();
    },
    attach(container: HTMLDivElement) {
      scene?.dispose();
      scene = createStageScene(container);
      invalidate();
      return scene;
    },
    resize(scale: number) {
      viewScale = scale;
      invalidate();
    },
    invalidate,
    /** The project itself changed: sound is rescheduled and newly referenced media decoded. */
    revise() {
      sound.revise();
      invalidate();
    },
    forget(asset: VideoAsset) {
      sources.forget(asset);
      audio.forget(asset);
      invalidate();
    },
    start() {
      sound.prepare();
      handle ??= requestAnimationFrame(tick);
    },
    stop() {
      if (handle !== undefined) cancelAnimationFrame(handle);
      handle = undefined;
      halt();
    },
    dispose() {
      if (handle !== undefined) cancelAnimationFrame(handle);
      handle = undefined;
      listeners.clear();
      controller.abort();
      sound.stop();
      sources.dispose();
      images.dispose();
      audio.dispose();
      videos.dispose();
      scene?.dispose();
      scene = undefined;
    },
  };
}
