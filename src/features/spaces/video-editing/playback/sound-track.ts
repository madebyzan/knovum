import type { AudioLibrary } from "../media/preview-audio";
import type { VideoEditProject } from "../project";
import { audioContext } from "./audio-context";
import { createAudioMixer, type AudioVoice } from "./audio-mixer";
import { buildVoices } from "./audio-voices";
import type { PlaybackClock } from "./playback-clock";

interface SoundTrackOptions {
  project: () => VideoEditProject | undefined;
  library: AudioLibrary;
  clock: PlaybackClock;
  onFailed: (error: unknown) => void;
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
 * The project's sound. Voices are decoded once per asset and scheduled against the clock they will
 * be heard on, so playback costs no per-frame audio work. An edit only re-schedules when it changes
 * what is audible, and a clip that finishes decoding late joins the timeline already in sync.
 */
export function createSoundTrack(options: SoundTrackOptions) {
  const mixer = createAudioMixer(audioContext);
  let scheduled = "";

  function voices() {
    const project = options.project();
    return project ? buildVoices(project, (asset) => options.library.ready(asset)) : [];
  }

  function schedule() {
    if (!options.clock.playing.value) return;
    const pending = voices();
    scheduled = signatureOf(pending);
    mixer.schedule(pending, {
      fromMs: options.clock.positionMs(),
      contextTimeOf: (timelineMs) => options.clock.contextTimeOf(timelineMs),
    });
  }

  function decode() {
    const project = options.project();
    for (const asset of project?.assets ?? []) {
      if (asset.kind === "image" || options.library.loaded(asset)) continue;
      void options.library.load(asset).then(schedule, options.onFailed);
    }
  }

  return {
    /** Decodes what the project needs and starts the voices that are already available. */
    prepare() {
      decode();
      schedule();
    },
    /** The position moved: every voice restarts from the new one. */
    reposition() {
      mixer.stop();
      schedule();
    },
    /** The project changed: only a different set of sounds is worth restarting for. */
    revise() {
      decode();
      if (signatureOf(voices()) === scheduled) return;
      mixer.stop();
      schedule();
    },
    stop() {
      mixer.stop();
      scheduled = "";
    },
  };
}
