import type { VideoAsset } from "../project";

export interface AudioVoice {
  key: string;
  asset: VideoAsset;
  sourceInMs: number;
  lengthMs: number;
  timelineStartMs: number;
  volume: number;
  /** Effective fades, already widened by any transition overlap the clip shares. */
  fadeInMs: number;
  fadeOutMs: number;
}

/** Where decoded source audio comes from: fixed-length pieces, read on demand. */
interface AudioPieces {
  readonly chunkMs: number;
  chunk(asset: VideoAsset, index: number): AudioBuffer | undefined;
  request(asset: VideoAsset, index: number): void;
}

export interface ScheduleWindow {
  /** The playhead: nothing before it is scheduled. */
  fromMs: number;
  /** How far ahead pieces are scheduled; beyond it they are only asked for. */
  untilMs: number;
  contextTimeOf: (timelineMs: number) => number;
}

interface Sounding {
  gain: GainNode;
  sources: Set<AudioBufferSourceNode>;
}

function levelAt(voice: AudioVoice, offsetMs: number) {
  const remaining = voice.lengthMs - offsetMs;
  const rise = voice.fadeInMs > 0 ? Math.min(1, offsetMs / voice.fadeInMs) : 1;
  const fall = voice.fadeOutMs > 0 ? Math.min(1, remaining / voice.fadeOutMs) : 1;
  return voice.volume * Math.max(0, Math.min(rise, fall));
}

/** The voice's stretch of source audio that falls inside the window, or nothing. */
function sourceSpan(voice: AudioVoice, window: ScheduleWindow) {
  const fromMs = Math.max(window.fromMs, voice.timelineStartMs);
  const untilMs = Math.min(window.untilMs, voice.timelineStartMs + voice.lengthMs);
  if (voice.volume <= 0 || fromMs >= untilMs) return undefined;
  const shift = voice.sourceInMs - voice.timelineStartMs;
  return { fromMs: fromMs + shift, untilMs: untilMs + shift };
}

/**
 * Schedules clip audio on the Web Audio timeline. Every voice gets one gain envelope, and its
 * source audio is played as consecutive decoded pieces started at the exact audio time they belong
 * to, so sound stays sample-accurate while only the pieces near the playhead are ever decoded. A
 * piece that arrives late joins at the position it would have reached, keeping picture and sound
 * on one clock.
 */
export function createAudioMixer(context: () => AudioContext, pieces: AudioPieces) {
  const soundingByVoiceKey = new Map<string, Sounding>();
  /** Pieces already handled per voice since the last stop, sounding or not. */
  const piecesByVoiceKey = new Map<string, Set<number>>();

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

  function soundingOf(voice: AudioVoice, startAt: number, contextTimeOf: (ms: number) => number) {
    const known = soundingByVoiceKey.get(voice.key);
    if (known) return known;
    const gain = envelope(voice, startAt, contextTimeOf);
    gain.connect(context().destination);
    const sounding: Sounding = { gain, sources: new Set() };
    soundingByVoiceKey.set(voice.key, sounding);
    return sounding;
  }

  function play(voice: AudioVoice, index: number, buffer: AudioBuffer, window: ScheduleWindow) {
    const audio = context();
    const chunkFromMs = index * pieces.chunkMs;
    const shift = voice.timelineStartMs - voice.sourceInMs;
    const fromMs = Math.max(chunkFromMs, window.fromMs - shift, voice.sourceInMs);
    const untilMs = Math.min(chunkFromMs + buffer.duration * 1000, voice.sourceInMs + voice.lengthMs);
    const scheduledAt = window.contextTimeOf(fromMs + shift);
    const lateS = Math.max(0, audio.currentTime - scheduledAt);
    const offsetS = (fromMs - chunkFromMs) / 1000 + lateS;
    const durationS = (untilMs - fromMs) / 1000 - lateS;
    if (durationS <= 0) return;
    const startAt = scheduledAt + lateS;
    const sounding = soundingOf(voice, startAt, window.contextTimeOf);
    const source = audio.createBufferSource();
    source.buffer = buffer;
    source.connect(sounding.gain);
    source.addEventListener(
      "ended",
      () => {
        source.disconnect();
        sounding.sources.delete(source);
      },
      { once: true },
    );
    source.start(startAt, offsetS, durationS);
    sounding.sources.add(source);
  }

  /** Every piece index of the voice's source audio the window covers. */
  function piecesIn(voice: AudioVoice, window: ScheduleWindow) {
    const span = sourceSpan(voice, window);
    if (!span) return [];
    const first = Math.floor(span.fromMs / pieces.chunkMs);
    const last = Math.ceil(span.untilMs / pieces.chunkMs) - 1;
    return Array.from({ length: Math.max(0, last - first + 1) }, (_, offset) => first + offset);
  }

  return {
    /** Starts every piece the window covers that is decoded and not sounding yet. */
    schedule(voices: readonly AudioVoice[], window: ScheduleWindow) {
      for (const voice of voices) {
        const handled = piecesByVoiceKey.get(voice.key) ?? new Set<number>();
        piecesByVoiceKey.set(voice.key, handled);
        for (const index of piecesIn(voice, window)) {
          if (handled.has(index)) continue;
          const buffer = pieces.chunk(voice.asset, index);
          if (!buffer) {
            pieces.request(voice.asset, index);
            continue;
          }
          play(voice, index, buffer, window);
          handled.add(index);
        }
      }
    },
    /** Asks for the pieces the window covers without playing them, so a start is immediate. */
    cue(voices: readonly AudioVoice[], window: ScheduleWindow) {
      for (const voice of voices) {
        for (const index of piecesIn(voice, window)) {
          if (!pieces.chunk(voice.asset, index)) pieces.request(voice.asset, index);
        }
      }
    },
    scheduled: (key: string) => soundingByVoiceKey.has(key),
    stop() {
      for (const sounding of soundingByVoiceKey.values()) {
        for (const source of sounding.sources) {
          source.stop();
          source.disconnect();
        }
        sounding.gain.disconnect();
      }
      soundingByVoiceKey.clear();
      piecesByVoiceKey.clear();
    },
  };
}
