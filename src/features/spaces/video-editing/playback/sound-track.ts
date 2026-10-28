import type { AudioChunks } from "../media/audio-chunks";
import type { VideoEditProject } from "../project";
import { audioContext } from "./audio-context";
import { createAudioMixer, type AudioVoice, type ScheduleWindow } from "./audio-mixer";
import { buildVoices } from "./audio-voices";
import type { PlaybackClock } from "./playback-clock";

/** How far ahead of the playhead sound is decoded and scheduled. */
const LOOKAHEAD_MS = 2_500;

interface SoundTrackOptions {
  project: () => VideoEditProject | undefined;
  chunks: AudioChunks;
  clock: PlaybackClock;
}

/** What makes a voice sound different; a transform or selection edit does not change it. */
function signatureOf(voices: AudioVoice[]) {
  return voices
    .map((voice) =>
      [
        voice.key,
        voice.sourceInMs,
        voice.lengthMs,
        voice.timelineStartMs,
        voice.volume,
        voice.fadeInMs,
        voice.fadeOutMs,
      ].join(":"),
    )
    .join("|");
}

/**
 * The project's sound. While playing, every frame keeps the next few seconds of every audible clip
 * decoded and scheduled against the clock they will be heard on; while paused, the same stretch
 * is decoded ahead so pressing play is heard at once. An edit only re-schedules when it changes
 * what is audible.
 */
export function createSoundTrack(options: SoundTrackOptions) {
  const mixer = createAudioMixer(audioContext, options.chunks);
  let scheduled = "";

  function voices() {
    const project = options.project();
    return project ? buildVoices(project) : [];
  }

  function window(): ScheduleWindow {
    const fromMs = options.clock.positionMs();
    return {
      fromMs,
      untilMs: fromMs + LOOKAHEAD_MS,
      contextTimeOf: (timelineMs) => options.clock.contextTimeOf(timelineMs),
    };
  }

  function pump() {
    const pending = voices();
    if (!options.clock.playing.value) return mixer.cue(pending, window());
    scheduled = signatureOf(pending);
    mixer.schedule(pending, window());
  }

  return {
    pump,
    /** The position moved: every voice restarts from the new one. */
    reposition() {
      mixer.stop();
      options.chunks.retarget();
      pump();
    },
    /** The project changed: only a different set of sounds is worth restarting for. */
    revise() {
      if (signatureOf(voices()) === scheduled) return pump();
      mixer.stop();
      pump();
    },
    stop() {
      mixer.stop();
      scheduled = "";
    },
  };
}
