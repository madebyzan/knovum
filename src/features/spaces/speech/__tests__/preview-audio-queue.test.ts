import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { SpeechPreviewAudioChunk } from "../../contracts/generated/SpeechPreviewAudioChunk";
import { PreviewAudioQueue } from "../preview-audio-queue";

const noop = () => undefined;

class FakeAudioBuffer {
  samples: Float32Array | undefined;

  copyToChannel(samples: Float32Array) {
    this.samples = samples;
  }
}

class FakeAudioSource {
  buffer: AudioBuffer | null = null;
  startedAt: number | undefined;
  stopped = false;
  private ended: EventListener | undefined;

  connect() {}
  disconnect() {}

  start(time?: number) {
    this.startedAt = time;
  }

  stop() {
    this.stopped = true;
  }

  addEventListener(_type: string, listener: EventListenerOrEventListenerObject) {
    this.ended = typeof listener === "function" ? listener : (event) => listener.handleEvent(event);
  }

  removeEventListener(_type: string, listener: EventListenerOrEventListenerObject) {
    if (this.ended === listener) this.ended = undefined;
  }

  finish() {
    this.ended?.(new Event("ended"));
  }
}

class FakeAudioContext {
  static instances: FakeAudioContext[] = [];
  static resumeGates: Promise<void>[] = [];

  state: AudioContextState = "suspended";
  currentTime = 0;
  readonly destination = {} as AudioDestinationNode;
  readonly sources: FakeAudioSource[] = [];
  readonly buffers: FakeAudioBuffer[] = [];
  private readonly resumeGate: Promise<void> | undefined;

  constructor() {
    this.resumeGate = FakeAudioContext.resumeGates.shift();
    FakeAudioContext.instances.push(this);
  }

  async resume() {
    await this.resumeGate;
    if (this.state !== "closed") this.state = "running";
  }

  async close() {
    this.state = "closed";
  }

  createBuffer() {
    const buffer = new FakeAudioBuffer();
    this.buffers.push(buffer);
    return buffer as unknown as AudioBuffer;
  }

  createBufferSource() {
    const source = new FakeAudioSource();
    this.sources.push(source);
    return source as unknown as AudioBufferSourceNode;
  }
}

function chunk(requestId: string, seq: number): SpeechPreviewAudioChunk {
  return {
    requestId,
    attemptEpoch: 0,
    seq,
    sampleRate: 8_000,
    sampleCount: 2,
    pcm16: [0, 0, 0xff, 0x7f],
  };
}

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

    const context = FakeAudioContext.instances[0];
    const source = context?.sources[0];
    const samples = context?.buffers[0]?.samples;
    expect(source?.startedAt).toBeCloseTo(0.03);
    expect(Array.from(samples ?? [])).toEqual([0, 32_767 / 32_768]);
    expect(playing).toEqual([false, true]);
    expect(() => queue.enqueue(chunk("request-1", 3))).toThrow("Preview audio chunk contract is invalid");

    source?.finish();
    expect(playing.at(-1)).toBe(false);
  });

  it("ignores late chunks and scopes stop to the owning request", async () => {
    const queue = new PreviewAudioQueue(() => undefined);
    expect(await queue.start("request-1")).toBe(true);
    expect(await queue.start("request-2")).toBe(true);

    queue.enqueue(chunk("request-1", 1));
    queue.stop("request-1");
    queue.enqueue(chunk("request-2", 1));
    expect(FakeAudioContext.instances[1]?.sources).toHaveLength(1);

    queue.stop("request-2");
    queue.enqueue(chunk("request-2", 2));
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
    expect(FakeAudioContext.instances[1]?.sources).toHaveLength(1);
    expect(FakeAudioContext.instances[0]?.state).toBe("closed");
  });
});
