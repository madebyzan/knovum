import { describe, expect, it, vi } from "vitest";
import { computed, effectScope } from "vue";

import { splitClip } from "../edits/arrange";
import { duplicateAudioClip } from "../edits/audio";
import { commitProject, shiftAfter } from "../edits/commit";
import { addTextOverlay, duplicateTextOverlay } from "../edits/text";
import { applyTransition } from "../edits/transition";
import { setClipWindow } from "../edits/trim";
import { createAudioMixer } from "../playback/audio-mixer";
import { createPlaybackClock } from "../playback/playback-clock";
import { createVideoEditProject, defaultTextStyle, type VideoEditProject } from "../project";
import { formatVideoTimelineTick } from "../timecode";

const noop = () => vi.fn<() => void>();

function sampleProject(): VideoEditProject {
  return createVideoEditProject({
    nodeId: "video-1",
    sourceRevision: "rev-1",
    durationMs: 10_000,
    sourceWidth: 1920,
    sourceHeight: 1080,
    frameRate: 30,
  });
}

describe("video-editing studio fixes", () => {
  it("invalidates reactive computeds when playback clock seeks while paused", () => {
    const scope = effectScope();
    const clock = createPlaybackClock({
      context: () => ({ currentTime: 0, resume: vi.fn<() => Promise<void>>() }) as unknown as AudioContext,
      durationMs: () => 10_000,
    });
    const pos = scope.run(() => computed(() => clock.positionMs()))!;
    expect(pos.value).toBe(0);
    clock.seek(4_200);
    expect(pos.value).toBe(4_200);
    scope.stop();
  });

  it("sorts audio clips per lane and prevents ripple overlap crashes in commitProject", () => {
    const base = sampleProject();
    const lateId = crypto.randomUUID();
    const earlyId = crypto.randomUUID();
    const withAudio: VideoEditProject = {
      ...base,
      assets: [
        ...base.assets,
        { nodeId: "audio-1", sourceRevision: "a1", kind: "audio", durationMs: 6_000, width: 0, height: 0 },
      ],
      audioTrack: {
        clips: [
          {
            id: lateId,
            assetId: "audio-1",
            sourceInMs: 0,
            sourceOutMs: 2_000,
            timelineStartMs: 5_000,
            lane: 0,
            volume: 1,
            fadeInMs: 0,
            fadeOutMs: 0,
          },
          {
            id: earlyId,
            assetId: "audio-1",
            sourceInMs: 0,
            sourceOutMs: 2_000,
            timelineStartMs: 1_000,
            lane: 0,
            volume: 1,
            fadeInMs: 0,
            fadeOutMs: 0,
          },
        ],
      },
    };
    const committed = commitProject(withAudio);
    expect(committed.audioTrack.clips.map((c) => c.id)).toEqual([earlyId, lateId]);

    // Left-shifting lateId into earlyId must resolve cleanly without throwing Zod overlap error.
    const shifted = commitProject(shiftAfter(committed, 4_000, -3_500));
    expect(shifted.audioTrack.clips[0]!.timelineStartMs).toBe(1_000);
    expect(shifted.audioTrack.clips[1]!.timelineStartMs).toBe(3_000);
  });

  it("prevents setClipWindow from breaking an outgoing transition length", () => {
    const split = splitClip(sampleProject(), 5_000);
    const firstId = split.videoTrack.clips[0]!.id;
    const withTransition = applyTransition(split, firstId, "dissolve", 500);
    const unchanged = setClipWindow(withTransition, firstId, { sourceOutMs: 5_500 });
    expect(unchanged).toBe(withTransition);
  });

  it("duplicates audio clips and text overlays cleanly", () => {
    const base = sampleProject();
    const withText = addTextOverlay(base, 1_000, defaultTextStyle("Inter"), "Hello");
    const textId = withText.textTrack.items[0]!.id;
    const dupText = duplicateTextOverlay(withText, textId);
    expect(dupText.textTrack.items).toHaveLength(2);
    expect(dupText.textTrack.items[1]!.startMs).toBe(4_000);

    const audioId = crypto.randomUUID();
    const withAudio = commitProject({
      ...base,
      assets: [
        ...base.assets,
        { nodeId: "audio-1", sourceRevision: "a1", kind: "audio", durationMs: 3_000, width: 0, height: 0 },
      ],
      audioTrack: {
        clips: [
          {
            id: audioId,
            assetId: "audio-1",
            sourceInMs: 0,
            sourceOutMs: 2_000,
            timelineStartMs: 0,
            lane: 0,
            volume: 1,
            fadeInMs: 0,
            fadeOutMs: 0,
          },
        ],
      },
    });
    const dupAudio = duplicateAudioClip(withAudio, audioId);
    expect(dupAudio.audioTrack.clips).toHaveLength(2);
    expect(dupAudio.audioTrack.clips[1]!.timelineStartMs).toBe(2_000);
  });

  it("does not overwrite interpolated gain with full volume when seeking inside fade-out", () => {
    const setValueAtTime = vi.fn<(value: number, time: number) => void>();
    const linearRampToValueAtTime = vi.fn<(value: number, time: number) => void>();
    const fakeContext = {
      currentTime: 0,
      destination: {},
      createGain: () => ({
        gain: { setValueAtTime, linearRampToValueAtTime },
        connect: noop(),
        disconnect: noop(),
      }),
      createBufferSource: () => ({
        buffer: null,
        connect: noop(),
        disconnect: noop(),
        addEventListener: noop(),
        start: noop(),
        stop: noop(),
      }),
    } as unknown as AudioContext;
    const mixer = createAudioMixer(() => fakeContext, {
      chunkMs: 5_000,
      chunk: () => ({ duration: 5 }) as AudioBuffer,
      request: noop(),
    });
    const asset = sampleProject().assets[0]!;
    mixer.schedule(
      [
        {
          key: "v1",
          asset,
          sourceInMs: 0,
          lengthMs: 4_000,
          timelineStartMs: 0,
          volume: 1,
          fadeInMs: 0,
          fadeOutMs: 2_000,
        },
      ],
      { fromMs: 3_000, untilMs: 4_000, contextTimeOf: (ms) => (ms - 3_000) / 1000 },
    );
    // At 3000ms (halfway through 2000ms fade-out), initial gain is 0.5 and must NOT be reset to 1.
    expect(setValueAtTime).toHaveBeenCalledTimes(1);
    expect(setValueAtTime).toHaveBeenCalledWith(0.5, 0);
    expect(linearRampToValueAtTime).toHaveBeenCalledWith(0, 1);
  });

  it("formats sub-second ruler ticks with tenths of a second", () => {
    expect(formatVideoTimelineTick(1_500, 500)).toBe("00:01.5");
    expect(formatVideoTimelineTick(1_000, 1_000)).toBe("00:01");
  });
});
