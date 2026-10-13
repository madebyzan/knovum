import { shallowRef } from "vue";

interface ClockOptions {
  context: () => AudioContext;
  durationMs: () => number;
}

/**
 * The audio clock is the only authority on playback position. Timeline time is anchored to the
 * audio context's own timeline, so scheduled sound and presented frames read the same ruler and the
 * picture can never drift away from what is audible.
 */
export function createPlaybackClock(options: ClockOptions) {
  const playing = shallowRef(false);
  let anchorMs = 0;
  let anchorContextS = 0;

  function positionMs() {
    if (!playing.value) return anchorMs;
    const elapsedMs = (options.context().currentTime - anchorContextS) * 1000;
    return Math.max(anchorMs, Math.min(anchorMs + elapsedMs, options.durationMs()));
  }

  return {
    playing,
    positionMs,
    /** Audio-context time at which a timeline position is heard; scheduling reads this. */
    contextTimeOf(timelineMs: number) {
      return anchorContextS + (timelineMs - anchorMs) / 1000;
    },
    async play() {
      if (playing.value) return;
      const context = options.context();
      await context.resume();
      anchorContextS = context.currentTime;
      playing.value = true;
    },
    pause() {
      if (!playing.value) return;
      anchorMs = positionMs();
      playing.value = false;
    },
    seek(timelineMs: number) {
      anchorMs = Math.max(0, timelineMs);
      if (playing.value) anchorContextS = options.context().currentTime;
    },
    endReached() {
      return playing.value && positionMs() >= options.durationMs();
    },
  };
}

export type PlaybackClock = ReturnType<typeof createPlaybackClock>;
