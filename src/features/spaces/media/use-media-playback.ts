import { useMediaControls } from "@vueuse/core";
import { clamp } from "es-toolkit";
import {
  computed,
  nextTick,
  onScopeDispose,
  shallowRef,
  toValue,
  watch,
  type MaybeRefOrGetter,
  type Ref,
} from "vue";

import { claimActiveMediaPlayback, releaseActiveMediaPlayback } from "./active-media-playback";
import { waitForMediaMetadata } from "./media-element-source";
import type { SpaceMediaPlaybackState } from "./playback";

const MEDIA_PLAYBACK_RATES = [0.75, 1, 1.25, 1.5, 2] as const;

interface UseMediaPlaybackOptions {
  sourceRevision?: MaybeRefOrGetter<string | number | undefined> | undefined;
  onPlayback?: ((state: SpaceMediaPlaybackState) => void) | undefined;
  onUnload?: (() => void) | undefined;
}

const boundAudioTrackElements = new WeakSet<HTMLMediaElement>();

function selectPrimaryAudioTrack(element: HTMLMediaElement) {
  const media = element as unknown as {
    audioTracks?: ArrayLike<{ enabled: boolean }> & {
      addEventListener?: (type: string, listener: () => void) => void;
    };
  };
  const enforce = () => {
    if (media.audioTracks && media.audioTracks.length > 1) {
      for (let index = 0; index < media.audioTracks.length; index++) {
        media.audioTracks[index]!.enabled = index === 0;
      }
    }
  };
  enforce();
  if (boundAudioTrackElements.has(element)) return;
  boundAudioTrackElements.add(element);
  media.audioTracks?.addEventListener?.("addtrack", enforce);
  media.audioTracks?.addEventListener?.("change", enforce);
}

export function useMediaPlayback(
  mediaElement: Ref<HTMLMediaElement | null | undefined>,
  options: UseMediaPlaybackOptions = {},
) {
  const loaded = shallowRef(false);
  const {
    playing,
    currentTime,
    duration: durationSeconds,
    rate,
    volume,
    muted,
  } = useMediaControls(mediaElement);

  const currentMs = computed(() => Math.round(currentTime.value * 1000));
  const durationMs = computed(() =>
    Number.isFinite(durationSeconds.value) ? Math.round(durationSeconds.value * 1000) : 0,
  );

  function pause() {
    playing.value = false;
    mediaElement.value?.pause();
    releaseActiveMediaPlayback(pause);
  }

  function emitPlayback() {
    if (!loaded.value) return;
    options.onPlayback?.({
      currentMs: currentMs.value,
      durationMs: durationMs.value,
      playing: playing.value,
    });
  }

  watch(playing, (isPlaying) => {
    if (isPlaying) claimActiveMediaPlayback(pause);
    else releaseActiveMediaPlayback(pause);
  });

  watch([currentMs, durationMs, playing], emitPlayback);

  watch(
    () => toValue(options.sourceRevision),
    () => unload(),
  );

  async function ensureReady() {
    loaded.value = true;
    await nextTick();
    const element = mediaElement.value;
    if (!element) throw new Error("Media element is not ready");
    if (element.readyState < HTMLMediaElement.HAVE_METADATA) {
      await waitForMediaMetadata(element, "Media metadata could not be loaded");
    }
    selectPrimaryAudioTrack(element);
    return element;
  }

  async function togglePlayback() {
    const element = await ensureReady();
    if (
      !playing.value &&
      (element.ended ||
        (Number.isFinite(element.duration) &&
          element.duration > 0 &&
          element.currentTime >= element.duration - 0.05))
    ) {
      element.currentTime = 0;
    }
    playing.value = !playing.value;
  }

  async function seekTo(positionMs: number, autoplay = true) {
    const element = await ensureReady();
    const maxSeconds =
      Number.isFinite(element.duration) && element.duration > 0 ? element.duration : Number.POSITIVE_INFINITY;
    element.currentTime = clamp(positionMs / 1000, 0, maxSeconds);
    if (!autoplay) return;
    if (playing.value) void element.play().catch(() => {});
    else playing.value = true;
  }

  function seekFromControl(positionMs: number) {
    const maxSeconds =
      Number.isFinite(durationSeconds.value) && durationSeconds.value > 0
        ? durationSeconds.value
        : Number.POSITIVE_INFINITY;
    currentTime.value = clamp(positionMs / 1000, 0, maxSeconds);
  }

  function skipBy(deltaMs: number) {
    seekFromControl(currentMs.value + deltaMs);
  }

  function cycleRate() {
    const currentIndex = MEDIA_PLAYBACK_RATES.findIndex(
      (candidate) => Math.abs(candidate - rate.value) < 0.01,
    );
    const next = MEDIA_PLAYBACK_RATES[(currentIndex + 1) % MEDIA_PLAYBACK_RATES.length] ?? 1;
    rate.value = next;
  }

  function toggleMute() {
    muted.value = !muted.value;
  }

  function unload() {
    pause();
    loaded.value = false;
    currentTime.value = 0;
    options.onUnload?.();
  }

  onScopeDispose(() => releaseActiveMediaPlayback(pause));

  return {
    loaded,
    playing,
    currentMs,
    durationMs,
    rate,
    volume,
    muted,
    ensureReady,
    togglePlayback,
    pause,
    seekTo,
    seekFromControl,
    skipBy,
    cycleRate,
    toggleMute,
    unload,
  };
}
