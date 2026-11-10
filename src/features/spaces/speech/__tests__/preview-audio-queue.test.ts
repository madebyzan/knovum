import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { SpeechPreviewAudioChunk } from "../../contracts/generated/SpeechPreviewAudioChunk";
import { PreviewAudioQueue } from "../preview-audio-queue";
import { FakeAudioContext } from "./fake-audio-context";

const noop = () => undefined;

const chunk = (requestId: string, seq: number): SpeechPreviewAudioChunk => ({
  requestId,
  attemptEpoch: 0,
  seq,
  sampleRate: 8_000,
  sampleCount: 2,
  pcm16: [0, 0, 0xff, 0x7f],
});

beforeEach(() => {
  FakeAudioContext.instances = [];
  FakeAudioContext.resumeGates = [];
  vi.stubGlobal("AudioContext", FakeAudioContext);
});

afterEach(() => vi.unstubAllGlobals());

describe("speech preview audio queue", () => {
  it("enforces sequence order and converts validated PCM16 bytes", async () => {
    const playing: boolean[] = [];
    const queue = new PreviewAudioQueue((value) => playing.push(value));

    expect(await queue.start("request-1")).toBe(true);
    queue.enqueue(chunk("request-1", 1));
    expect(() => queue.enqueue(chunk("request-1", 3))).toThrow("Preview audio chunk contract is invalid");
    queue.flush();

    const context = FakeAudioContext.instances[0];
    const source = context?.sources[0];
    const samples = context?.buffers[0]?.samples;
    expect(source?.startedAt).toBeCloseTo(0.03);
    expect(Array.from(samples ?? [])).toEqual([0, 32_767 / 32_768]);
    expect(playing).toEqual([false, true]);

    source?.finish();
    expect(playing.at(-1)).toBe(false);
  });

  it("buffers successive chunks during generation and plays them smoothly on flush", async () => {
    const queue = new PreviewAudioQueue(() => undefined);
    expect(await queue.start("request-1")).toBe(true);

    const context = FakeAudioContext.instances[0]!;
    queue.enqueue(chunk("request-1", 1));
    queue.enqueue(chunk("request-1", 2));
    queue.enqueue(chunk("request-1", 3));
    expect(context.sources).toHaveLength(0);

    queue.flush();
    expect(context.sources).toHaveLength(3);
  });

  it("ignores late chunks and scopes stop to the owning request", async () => {
    const queue = new PreviewAudioQueue(() => undefined);
    expect(await queue.start("request-1")).toBe(true);
    expect(await queue.start("request-2")).toBe(true);

    queue.enqueue(chunk("request-1", 1));
    queue.stop("request-1");
    queue.enqueue(chunk("request-2", 1));
    queue.flush();
    expect(FakeAudioContext.instances[1]?.sources).toHaveLength(1);

    queue.stop("request-2");
    queue.enqueue(chunk("request-2", 2));
    queue.flush();
    expect(FakeAudioContext.instances[1]?.sources).toHaveLength(1);
    expect(FakeAudioContext.instances[1]?.state).toBe("closed");
  });

  it("fences an older start that resumes after a newer session", async () => {
    let releaseFirst: () => void = noop;
    FakeAudioContext.resumeGates.push(
      new Promise<void>((resolve) => {
        releaseFirst = resolve;
      }),
      Promise.resolve(),
    );
    const queue = new PreviewAudioQueue(() => undefined);
    const first = queue.start("request-1");
    await Promise.resolve();
    expect(await queue.start("request-2")).toBe(true);

    releaseFirst();
    expect(await first).toBe(false);
    queue.enqueue(chunk("request-2", 1));
    queue.flush();
    expect(FakeAudioContext.instances[1]?.sources).toHaveLength(1);
    expect(FakeAudioContext.instances[0]?.state).toBe("closed");
  });
});

it("reports the amplitude of the audio being heard and silence once it stops", async () => {
  const queue = new PreviewAudioQueue(() => {});
  await queue.start("job");
  const context = FakeAudioContext.instances[0]!;
  const analyser = context.analysers[0]!;

  // Nothing is scheduled yet: a waveform must draw a flat line rather than invent movement.
  analyser.amplitude = 1;
  expect(queue.peak()).toBe(0);

  queue.enqueue(chunk("job", 1));
  expect(queue.peak()).toBe(0);

  queue.flush();
  analyser.amplitude = 0.5;
  expect(queue.peak()).toBeCloseTo(0.5, 1);

  queue.stop();
  expect(queue.peak()).toBe(0);
});

it("waits until all scheduled chunks finish playing", async () => {
  const queue = new PreviewAudioQueue(() => {});
  await queue.start("job");
  const context = FakeAudioContext.instances[0]!;

  queue.enqueue(chunk("job", 1));
  queue.flush();
  let finished = false;
  const waiting = queue.waitUntilEnded().then(() => {
    finished = true;
    return true;
  });

  expect(finished).toBe(false);
  context.sources[0]!.finish();
  await waiting;
  expect(finished).toBe(true);
});
