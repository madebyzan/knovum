import { describe, expect, it, vi } from "vitest";

import { insertMaterial } from "../edits/material";
import { applyTransition } from "../edits/transition";
import { createAudioMixer } from "../playback/audio-mixer";
import { buildVoices } from "../playback/audio-voices";
import { createPlaybackClock } from "../playback/playback-clock";
import { createVideoEditProject, type VideoAsset } from "../project";

const music: VideoAsset = {
  nodeId: "music",
  sourceRevision: "music-revision",
  kind: "audio",
  durationMs: 30_000,
  width: 0,
  height: 0,
};
const buffer = { duration: 4, length: 192_000, numberOfChannels: 2 } as AudioBuffer;

function project() {
  return createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 4_000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
}

function fakeContext() {
  const context = {
    currentTime: 0,
    resume: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  };
  return context;
}

describe("playback clock", () => {
  it("reads the position from the audio timeline while playing and freezes when paused", async () => {
    const context = fakeContext();
    const clock = createPlaybackClock({
      context: () => context as unknown as AudioContext,
      durationMs: () => 4_000,
    });
    clock.seek(500);
    expect(clock.positionMs()).toBe(500);
    await clock.play();
    context.currentTime = 0.25;
    expect(clock.positionMs()).toBe(750);
    clock.pause();
    context.currentTime = 2;
    expect(clock.positionMs()).toBe(750);
    expect(context.resume).toHaveBeenCalledOnce();
  });

  it("never runs past the end of the project and says so", async () => {
    const context = fakeContext();
    const clock = createPlaybackClock({
      context: () => context as unknown as AudioContext,
      durationMs: () => 1_000,
    });
    await clock.play();
    context.currentTime = 5;
    expect(clock.positionMs()).toBe(1_000);
    expect(clock.endReached()).toBe(true);
  });

  it("maps a timeline position to the audio time it will be heard at", async () => {
    const context = fakeContext();
    const clock = createPlaybackClock({
      context: () => context as unknown as AudioContext,
      durationMs: () => 10_000,
    });
    context.currentTime = 3;
    await clock.play();
    clock.seek(1_000);
    expect(clock.contextTimeOf(1_500)).toBeCloseTo(3.5, 6);
    expect(clock.contextTimeOf(0)).toBeCloseTo(2, 6);
  });
});

describe("project voices", () => {
  it("widens a clip's fades by the transition overlap it shares, like the export", () => {
    const initial = insertMaterial(project(), { ...music, kind: "video", width: 720, height: 1280 }, 4_000);
    const mixed = applyTransition(initial, initial.videoTrack.clips[0]!.id, "dissolve", 1_000);
    const [first, second] = buildVoices(mixed, () => buffer);
    expect(first).toMatchObject({ fadeOutMs: 1_000, timelineStartMs: 0 });
    expect(second).toMatchObject({ fadeInMs: 1_000 });
  });

  it("leaves out assets whose audio has not been decoded", () => {
    const withMusic = insertMaterial(project(), music, 0);
    const voices = buildVoices(withMusic, (asset) => (asset.kind === "audio" ? buffer : undefined));
    expect(voices.map(({ key }) => key)).toEqual([withMusic.audioTrack.clips[0]!.id]);
  });
});

describe("audio mixer", () => {
  it("starts every voice once, at the audio time and source offset it belongs to", () => {
    const sources: { when: number; offset: number; duration: number }[] = [];
    const context = {
      currentTime: 10,
      destination: {},
      createGain: () => ({
        gain: {
          setValueAtTime: vi.fn<() => void>(),
          linearRampToValueAtTime: vi.fn<() => void>(),
        },
        connect: vi.fn<() => void>(),
        disconnect: vi.fn<() => void>(),
      }),
      createBufferSource: () => ({
        buffer: undefined,
        addEventListener: vi.fn<() => void>(),
        removeEventListener: vi.fn<() => void>(),
        connect: vi.fn<() => void>(),
        disconnect: vi.fn<() => void>(),
        stop: vi.fn<() => void>(),
        start: (when: number, offset: number, duration: number) => sources.push({ when, offset, duration }),
      }),
    };
    const mixer = createAudioMixer(() => context as unknown as AudioContext);
    const voice = {
      key: "clip",
      buffer,
      sourceInMs: 500,
      lengthMs: 2_000,
      timelineStartMs: 1_000,
      volume: 1,
      fadeInMs: 0,
      fadeOutMs: 0,
    };
    const position = { fromMs: 1_500, contextTimeOf: (ms: number) => 10 + (ms - 1_500) / 1000 };
    mixer.schedule([voice], position);
    mixer.schedule([voice], position);
    expect(sources).toEqual([{ when: 10, offset: 1, duration: 1.5 }]);
    expect(mixer.scheduled("clip")).toBe(true);
    mixer.stop();
    expect(mixer.scheduled("clip")).toBe(false);
  });

  it("ignores a silent voice and one the position has already passed", () => {
    const context = {
      currentTime: 0,
      destination: {},
      createGain: vi.fn<() => GainNode>(),
      createBufferSource: vi.fn<() => AudioBufferSourceNode>(),
    };
    const mixer = createAudioMixer(() => context as unknown as AudioContext);
    const base = { buffer, sourceInMs: 0, lengthMs: 1_000, timelineStartMs: 0, fadeInMs: 0, fadeOutMs: 0 };
    mixer.schedule(
      [
        { ...base, key: "silent", volume: 0 },
        { ...base, key: "past", volume: 1 },
      ],
      { fromMs: 5_000, contextTimeOf: (ms: number) => ms / 1000 },
    );
    expect(context.createBufferSource).not.toHaveBeenCalled();
  });
});
