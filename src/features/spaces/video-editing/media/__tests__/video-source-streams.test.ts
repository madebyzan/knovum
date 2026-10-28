import { beforeEach, describe, expect, it, vi } from "vitest";

import type { VideoAsset } from "../../project";
import { waveformBars } from "../../timeline/waveform";
import { AUDIO_CHUNK_MS, audioBufferOf, createAudioChunks } from "../audio-chunks";
import { previewFrameSize } from "../preview-video";

const ipc = vi.hoisted(() => ({ readSourceAudio: vi.fn<() => Promise<ArrayBuffer>>() }));
vi.mock("../source-ipc", () => ({
  mediaSourceOf: (spaceId: string, { nodeId, sourceRevision }: VideoAsset) => ({
    spaceId,
    sourceNodeId: nodeId,
    sourceRevision,
  }),
  readSourceAudio: ipc.readSourceAudio,
}));

const asset: VideoAsset = {
  nodeId: "source",
  sourceRevision: "revision",
  kind: "video",
  durationMs: 12_000,
  width: 3840,
  height: 2160,
};

function fakeContext() {
  return {
    sampleRate: 4,
    createBuffer: (channels: number, frames: number, sampleRate: number) => {
      const data = Array.from({ length: channels }, () => new Float32Array(frames));
      return {
        numberOfChannels: channels,
        length: frames,
        sampleRate,
        duration: frames / sampleRate,
        copyToChannel: (source: Float32Array, channel: number) => data[channel]!.set(source),
        getChannelData: (channel: number) => data[channel]!,
      };
    },
  } as unknown as AudioContext;
}

const stereo = (...samples: number[]) => Float32Array.from(samples).buffer;
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("source audio pieces", () => {
  beforeEach(() => {
    ipc.readSourceAudio.mockReset();
  });

  it("de-interleaves stereo samples into one channel each", () => {
    const buffer = audioBufferOf(fakeContext(), stereo(0.1, -0.1, 0.2, -0.2))!;
    expect([...buffer.getChannelData(0)]).toEqual([0.1, 0.2].map(Math.fround));
    expect([...buffer.getChannelData(1)]).toEqual([-0.1, -0.2].map(Math.fround));
    expect(audioBufferOf(fakeContext(), new ArrayBuffer(0))).toBeUndefined();
  });

  it("reads each piece once, only up to the end of the source", async () => {
    ipc.readSourceAudio.mockResolvedValue(stereo(0.5, 0.5));
    const chunks = createAudioChunks({
      spaceId: "space",
      context: fakeContext,
      audible: () => Promise.resolve(true),
      onFailed: vi.fn<(error: unknown) => void>(),
    });
    chunks.request(asset, 2);
    chunks.request(asset, 2);
    chunks.request(asset, 3);
    await settle();
    expect(ipc.readSourceAudio).toHaveBeenCalledOnce();
    expect(ipc.readSourceAudio).toHaveBeenCalledWith(
      { spaceId: "space", sourceNodeId: "source", sourceRevision: "revision" },
      { fromMs: 2 * AUDIO_CHUNK_MS, lengthMs: 2_000, sampleRate: 4 },
    );
    expect(chunks.chunk(asset, 2)?.length).toBe(1);
  });

  it("never asks again for a source without sound", async () => {
    const silent = chunksWith(() => Promise.resolve(false));
    silent.request(asset, 0);
    await settle();
    silent.request(asset, 1);
    await settle();
    expect(ipc.readSourceAudio).not.toHaveBeenCalled();
  });

  it("reports a failed read once and does not retry it until asked to", async () => {
    ipc.readSourceAudio.mockImplementation(() => Promise.reject(new Error("unreadable")));
    const onFailed = vi.fn<(error: unknown) => void>();
    const broken = chunksWith(() => Promise.resolve(true), onFailed);
    broken.request(asset, 0);
    broken.request(asset, 1);
    await settle();
    broken.request(asset, 0);
    await settle();
    expect(onFailed).toHaveBeenCalledOnce();
    expect(ipc.readSourceAudio).toHaveBeenCalledTimes(2);
    broken.retry();
    broken.request(asset, 0);
    await settle();
    expect(ipc.readSourceAudio).toHaveBeenCalledTimes(3);
  });
});

function chunksWith(audible: () => Promise<boolean>, onFailed = vi.fn<(error: unknown) => void>()) {
  return createAudioChunks({ spaceId: "space", context: fakeContext, audible, onFailed });
}

describe("preview frame size", () => {
  it("decodes at the source's own size up to the preview edge, keeping its shape", () => {
    expect(previewFrameSize(1920, 1080)).toEqual({});
    expect(previewFrameSize(3840, 2160)).toEqual({ width: 1920, height: 1080, fit: "fill" });
    expect(previewFrameSize(2160, 3840)).toEqual({ width: 1080, height: 1920, fit: "fill" });
  });
});

describe("waveform bars", () => {
  it("takes the loudest peak of each slice of the source window", () => {
    const waveform = { rate: 10, peaks: Uint8Array.from([0, 255, 51, 102, 0, 0]) };
    expect(waveformBars(waveform, { fromMs: 0, toMs: 400 }, 2)).toEqual([1, 0.4]);
    expect(waveformBars(waveform, { fromMs: 200, toMs: 600 }, 1)).toEqual([0.4]);
  });

  it("reads the part not decoded yet as silence", () => {
    const waveform = { rate: 10, peaks: new Uint8Array(4) };
    expect(waveformBars(waveform, { fromMs: 0, toMs: 400 }, 4)).toEqual([0, 0, 0, 0]);
    expect(waveformBars(undefined, { fromMs: 0, toMs: 400 }, 4)).toEqual([]);
  });
});
