/**
 * A single Web Audio context for the whole preview. One context per media element starves the
 * browser's context budget and gives every element its own clock, which is what made preview audio
 * drift away from the picture.
 */
export function createAudioGraph() {
  let context: AudioContext | undefined;
  const gains = new WeakMap<HTMLMediaElement, GainNode>();

  function ensure() {
    context ??= new AudioContext();
    return context;
  }

  function attach(element: HTMLMediaElement) {
    const existing = gains.get(element);
    if (existing) return existing;
    const audio = ensure();
    const gain = audio.createGain();
    audio.createMediaElementSource(element).connect(gain).connect(audio.destination);
    gains.set(element, gain);
    return gain;
  }

  return {
    setGain(element: HTMLMediaElement, value: number) {
      const gain = attach(element);
      const target = Math.max(0, Math.min(2, value));
      if (Math.abs(gain.gain.value - target) > 0.001) gain.gain.value = target;
    },
    async resume() {
      const audio = ensure();
      if (audio.state === "suspended") await audio.resume();
    },
    dispose() {
      void context?.close();
      context = undefined;
    },
  };
}

export type AudioGraph = ReturnType<typeof createAudioGraph>;
