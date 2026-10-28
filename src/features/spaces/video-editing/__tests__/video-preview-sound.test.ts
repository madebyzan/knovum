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
    const [first, second] = buildVoices(mixed);
    expect(first).toMatchObject({ fadeOutMs: 1_000, timelineStartMs: 0 });
    expect(second).toMatchObject({ fadeInMs: 1_000 });
  });

  it("gives a still image no voice", () => {
    const withStill = insertMaterial(project(), { ...music, kind: "image", width: 720, height: 1280 }, 4_000);
    expect(buildVoices(withStill).map(({ asset }) => asset.kind)).toEqual(["video"]);
  });
});

function fakeAudio(currentTime: number) {
  const started: { when: number; offset: number; duration: number }[] = [];
  const context = {
    currentTime,
    destination: {},
    createGain: () => ({
      gain: { setValueAtTime: vi.fn<() => void>(), linearRampToValueAtTime: vi.fn<() => void>() },
      connect: vi.fn<() => void>(),
      disconnect: vi.fn<() => void>(),
    }),
    createBufferSource: () => ({
      buffer: undefined,
      addEventListener: vi.fn<() => void>(),
      connect: vi.fn<() => void>(),
      disconnect: vi.fn<() => void>(),
      stop: vi.fn<() => void>(),
      start: (when: number, offset: number, duration: number) => started.push({ when, offset, duration }),
    }),
  };
  return { context: () => context as unknown as AudioContext, started };
}

function pieces(ready: (index: number) => boolean) {
  const piece = { duration: 1 } as AudioBuffer;
  return {
    chunkMs: 1_000,
    chunk: (_asset: VideoAsset, index: number) => (ready(index) ? piece : undefined),
    request: vi.fn<(asset: VideoAsset, index: number) => void>(),
  };
}

/** Audio time of a timeline position when 1.5 s is heard at 10 s. */
const contextTimeOf = (ms: number) => 10 + (ms - 1_500) / 1000;

const voice = {
  key: "clip",
  asset: music,
  sourceInMs: 500,
  lengthMs: 2_000,
  timelineStartMs: 1_000,
  volume: 1,
  fadeInMs: 0,
  fadeOutMs: 0,
};

describe("audio mixer", () => {
  it("plays consecutive pieces at the audio time and source offset each belongs to", () => {
    const audio = fakeAudio(10);
    const mixer = createAudioMixer(
      audio.context,
      pieces(() => true),
    );
    const window = { fromMs: 1_500, untilMs: 4_000, contextTimeOf };
    mixer.schedule([voice], window);
    mixer.schedule([voice], window);
    expect(audio.started).toEqual([
      { when: 10, offset: 0, duration: 1 },
      { when: 11, offset: 0, duration: 0.5 },
    ]);
    expect(mixer.scheduled("clip")).toBe(true);
    mixer.stop();
    expect(mixer.scheduled("clip")).toBe(false);
  });

  it("asks for a piece that is not decoded yet and joins it in sync once it is", () => {
    const audio = fakeAudio(10);
    let decoded = false;
    const source = pieces(() => decoded);
    const mixer = createAudioMixer(audio.context, source);
    mixer.schedule([voice], { fromMs: 1_500, untilMs: 2_000, contextTimeOf });
    expect(source.request).toHaveBeenCalledWith(music, 1);
    expect(audio.started).toEqual([]);
    decoded = true;
    (audio.context() as { currentTime: number }).currentTime = 10.25;
    mixer.schedule([voice], { fromMs: 1_750, untilMs: 2_250, contextTimeOf });
    expect(audio.started[0]).toMatchObject({ when: 10.25 });
    expect(audio.started[0]!.offset).toBeCloseTo(0.25, 6);
    expect(audio.started[0]!.duration).toBeCloseTo(0.75, 6);
  });

  it("ignores a silent voice and one the window has already passed", () => {
    const audio = fakeAudio(0);
    const source = pieces(() => true);
    const mixer = createAudioMixer(audio.context, source);
    mixer.schedule(
      [
        { ...voice, key: "silent", volume: 0 },
        { ...voice, key: "past" },
      ],
      {
        fromMs: 5_000,
        untilMs: 7_000,
        contextTimeOf: (ms: number) => ms / 1000,
      },
    );
    expect(audio.started).toEqual([]);
    expect(source.request).not.toHaveBeenCalled();
  });
});
