interface Routed {
  source: MediaElementAudioSourceNode;
  gain: GainNode;
}

/**
 * A single Web Audio context for the whole preview. One context per media element starves the
 * browser's context budget and gives every element its own clock, which is what made preview audio
 * drift away from the picture. The same context decodes waveform data, so the timeline never opens
 * a second one.
 */
export function createAudioGraph() {
  let context: AudioContext | undefined;
  const routed = new WeakMap<HTMLMediaElement, Routed>();

  function ensure() {
    context ??= new AudioContext();
    return context;
  }

  function attach(element: HTMLMediaElement) {
    const existing = routed.get(element);
    if (existing) return existing.gain;
    const audio = ensure();
    const gain = audio.createGain();
    const source = audio.createMediaElementSource(element);
    source.connect(gain).connect(audio.destination);
    routed.set(element, { source, gain });
    return gain;
  }

  return {
    setGain(element: HTMLMediaElement, value: number) {
      const gain = attach(element);
      const target = Math.max(0, Math.min(2, value));
      if (Math.abs(gain.gain.value - target) > 0.001) gain.gain.value = target;
    },
    /** Frees the graph nodes of a released element; an orphaned source node keeps it alive. */
    detach(element: HTMLMediaElement) {
      const existing = routed.get(element);
      if (!existing) return;
      existing.source.disconnect();
      existing.gain.disconnect();
      routed.delete(element);
    },
    decode(bytes: ArrayBuffer) {
      return ensure().decodeAudioData(bytes);
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
