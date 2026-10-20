export interface AudioVoice {
  key: string;
  buffer: AudioBuffer;
  sourceInMs: number;
  lengthMs: number;
  timelineStartMs: number;
  volume: number;
  /** Effective fades, already widened by any transition overlap the clip shares. */
  fadeInMs: number;
  fadeOutMs: number;
}

interface Playing {
  source: AudioBufferSourceNode;
  gain: GainNode;
  release: () => void;
}

function levelAt(voice: AudioVoice, offsetMs: number) {
  const remaining = voice.lengthMs - offsetMs;
  const rise = voice.fadeInMs > 0 ? Math.min(1, offsetMs / voice.fadeInMs) : 1;
  const fall = voice.fadeOutMs > 0 ? Math.min(1, remaining / voice.fadeOutMs) : 1;
  return voice.volume * Math.max(0, Math.min(rise, fall));
}

/**
 * Schedules clip audio on the Web Audio timeline. Every voice is started once with its own gain
 * envelope, so sound is sample-accurate and costs nothing per frame; the picture follows the clock
 * these voices were scheduled against.
 */
export function createAudioMixer(context: () => AudioContext) {
  const playingByVoiceKey = new Map<string, Playing>();

  function envelope(voice: AudioVoice, startAt: number, contextTimeOf: (ms: number) => number) {
    const gain = context().createGain();
    const endMs = voice.timelineStartMs + voice.lengthMs;
    const offsetMs = Math.max(0, (startAt - contextTimeOf(voice.timelineStartMs)) * 1000);
    gain.gain.setValueAtTime(levelAt(voice, offsetMs), startAt);
    const risenAt = Math.max(startAt, contextTimeOf(voice.timelineStartMs + voice.fadeInMs));
    if (voice.fadeInMs > 0 && risenAt > startAt) gain.gain.linearRampToValueAtTime(voice.volume, risenAt);
    if (voice.fadeOutMs > 0) {
      const fallsAt = Math.max(risenAt, contextTimeOf(endMs - voice.fadeOutMs));
      gain.gain.setValueAtTime(voice.volume, fallsAt);
      const endsAt = Math.max(fallsAt, contextTimeOf(endMs));
      if (endsAt > fallsAt) gain.gain.linearRampToValueAtTime(0, endsAt);
    }
    return gain;
  }

  function start(voice: AudioVoice, fromMs: number, contextTimeOf: (ms: number) => number) {
    const endMs = voice.timelineStartMs + voice.lengthMs;
    const playFromMs = Math.max(fromMs, voice.timelineStartMs);
    if (playingByVoiceKey.has(voice.key) || playFromMs >= endMs || voice.volume <= 0) return;
    const offsetS = Math.max(0, (voice.sourceInMs + (playFromMs - voice.timelineStartMs)) / 1000);
    if (offsetS >= voice.buffer.duration) return;
    const durationS = Math.min((endMs - playFromMs) / 1000, voice.buffer.duration - offsetS);
    if (durationS <= 0) return;
    const audio = context();
    const startAt = Math.max(audio.currentTime, contextTimeOf(playFromMs));
    const gain = envelope(voice, startAt, contextTimeOf);
    gain.connect(audio.destination);
    const source = audio.createBufferSource();
    source.buffer = voice.buffer;
    source.connect(gain);
    const release = () => {
      gain.disconnect();
      if (playingByVoiceKey.get(voice.key)?.source === source) playingByVoiceKey.delete(voice.key);
    };
    source.addEventListener("ended", release, { once: true });
    source.start(startAt, offsetS, durationS);
    playingByVoiceKey.set(voice.key, { source, gain, release });
  }

  function stop() {
    for (const voice of playingByVoiceKey.values()) {
      voice.source.removeEventListener("ended", voice.release);
      voice.source.stop();
      voice.source.disconnect();
      voice.gain.disconnect();
    }
    playingByVoiceKey.clear();
  }

  return {
    /** Starts every voice that is still ahead of the position and not already sounding. */
    schedule(voices: AudioVoice[], position: { fromMs: number; contextTimeOf: (ms: number) => number }) {
      for (const voice of voices) start(voice, position.fromMs, position.contextTimeOf);
    },
    scheduled: (key: string) => playingByVoiceKey.has(key),
    stop,
  };
}
