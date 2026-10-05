import { useMediaControls } from "@vueuse/core";
import { clamp } from "es-toolkit";
import { computed, nextTick, shallowRef, toValue, watch, type MaybeRefOrGetter, type Ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { waitForMediaMetadata } from "./media-element-source";
import type { SpaceMediaPlaybackState } from "./playback";

interface UseMediaPlaybackOptions {
  sourceRevision?: MaybeRefOrGetter<string | number | undefined> | undefined;
  onPlayback?: ((state: SpaceMediaPlaybackState) => void) | undefined;
  onPlaybackError?: ((message: string) => void) | undefined;
  onUnload?: (() => void) | undefined;
}

export function useMediaPlayback(
  mediaElement: Ref<HTMLMediaElement | null | undefined>,
  options: UseMediaPlaybackOptions = {},
) {
  const loaded = shallowRef(false);
  const controls = useMediaControls(mediaElement);

  const playing = computed({
    get: () => controls.playing.value,
    set: (v) => {
      controls.playing.value = v;
    },
  });

  const currentMs = computed(() => Math.round(controls.currentTime.value * 1000));
  const durationMs = computed(() =>
    Number.isFinite(controls.duration.value) ? Math.round(controls.duration.value * 1000) : 0,
  );

  function emitPlayback() {
    if (!loaded.value) return;
    options.onPlayback?.({
      currentMs: currentMs.value,
      durationMs: durationMs.value,
      playing: playing.value,
    });
  }

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
    return element;
  }

  async function togglePlayback() {
    try {
      await ensureReady();
      playing.value = !playing.value;
    } catch (error) {
      options.onPlaybackError?.(getErrorMessage(error));
    }
  }

  async function seekTo(positionMs: number) {
    const element = await ensureReady();
    element.currentTime = clamp(positionMs / 1000, 0, element.duration || Number.POSITIVE_INFINITY);
    playing.value = true;
  }

  function seekFromControl(positionMs: number) {
    controls.currentTime.value = positionMs / 1000;
  }

  function unload() {
    playing.value = false;
    loaded.value = false;
    controls.currentTime.value = 0;
    options.onUnload?.();
  }

  return {
    loaded,
    playing,
    currentMs,
    durationMs,
    ensureReady,
    togglePlayback,
    seekTo,
    seekFromControl,
    unload,
  };
}
