import { shallowRef } from "vue";

const BAR_COUNT = 12;
const IDLE_LEVELS = Array.from({ length: BAR_COUNT }, () => 0);

export interface AudioLevelMeter {
  stop: () => void;
}

function createAudioLevelMeter(stream: MediaStream, onLevels: (levels: number[]) => void): AudioLevelMeter {
  const AudioCtx =
    globalThis.AudioContext ??
    (globalThis as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AudioCtx) {
    onLevels(IDLE_LEVELS);
    return { stop: () => {} };
  }
  let context: AudioContext | undefined;
  let rafId = 0;
  let stopped = false;
  try {
    context = new AudioCtx();
    const source = context.createMediaStreamSource(stream);
    const analyser = context.createAnalyser();
    analyser.fftSize = 256;
    analyser.smoothingTimeConstant = 0.72;
    source.connect(analyser);
    const data = new Uint8Array(analyser.frequencyBinCount);
    const usableBins = Math.min(data.length, 48);
    const step = Math.max(1, Math.floor(usableBins / BAR_COUNT));
    const sample = () => {
      if (stopped) return;
      analyser.getByteFrequencyData(data);
      const bars = Array.from({ length: BAR_COUNT }, (_, index) => {
        let sum = 0;
        const start = index * step;
        for (let offset = 0; offset < step; offset++) sum += data[start + offset] ?? 0;
        return Math.min(1, sum / (step * 180));
      });
      onLevels(bars);
      rafId = globalThis.requestAnimationFrame(sample);
    };
    rafId = globalThis.requestAnimationFrame(sample);
  } catch {
    onLevels(IDLE_LEVELS);
  }
  return {
    stop() {
      stopped = true;
      if (rafId) globalThis.cancelAnimationFrame(rafId);
      onLevels(IDLE_LEVELS);
      if (context && context.state !== "closed") void context.close().catch(() => {});
    },
  };
}

export function useAudioLevelMeter() {
  const levels = shallowRef<number[]>(IDLE_LEVELS);
  let active: AudioLevelMeter | undefined;

  function start(stream: MediaStream, isPaused: () => boolean) {
    stop();
    active = createAudioLevelMeter(stream, (next) => {
      levels.value = isPaused() ? IDLE_LEVELS : next;
    });
  }

  function clear() {
    levels.value = IDLE_LEVELS;
  }

  function stop() {
    active?.stop();
    active = undefined;
    levels.value = IDLE_LEVELS;
  }

  return { levels, start, clear, stop };
}
