import { shallowRef } from "vue";

interface PlaybackClockOptions {
  durationMs: () => number;
  /** Last position that can be shown, so the playhead never rests past the final frame. */
  lastFrameMs: () => number;
}

/** Longest step a single frame may advance; a throttled or stalled frame must not jump the clock. */
const MAX_STEP_MS = 120;

/**
 * The timeline clock is the single playback authority. Media elements are corrected towards it, and
 * it stops advancing while they buffer, so a slow decode shows as a short wait instead of a jumping
 * picture.
 */
export function createPlaybackClock(options: PlaybackClockOptions) {
  const timeMs = shallowRef(0);
  const playing = shallowRef(false);
  const stalled = shallowRef(false);
  let previousTick: number | undefined;

  function seek(value: number) {
    timeMs.value = Math.max(0, Math.min(value, options.durationMs()));
  }

  function pause() {
    playing.value = false;
    stalled.value = false;
    previousTick = undefined;
  }

  function play() {
    if (timeMs.value >= options.lastFrameMs()) seek(0);
    playing.value = true;
    previousTick = undefined;
  }

  function tick(now: number, ready: boolean) {
    if (!playing.value) {
      previousTick = undefined;
      return;
    }
    stalled.value = !ready;
    const previous = previousTick;
    previousTick = now;
    if (!ready || previous === undefined) return;
    const advanced = timeMs.value + Math.min(now - previous, MAX_STEP_MS);
    if (advanced >= options.durationMs()) {
      seek(options.lastFrameMs());
      pause();
      return;
    }
    seek(advanced);
  }

  return {
    timeMs,
    playing,
    stalled,
    play,
    pause,
    tick,
    seek(value: number) {
      seek(Math.min(value, options.lastFrameMs()));
    },
    toggle() {
      if (playing.value) pause();
      else play();
    },
  };
}
