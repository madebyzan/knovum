import type { CanvasSink } from "mediabunny";
import { describe, expect, it } from "vitest";

import { createFrameFeed } from "../video-frames";

interface Frame {
  id: string;
  timestamp: number;
  duration: number;
}

const flush = () => new Promise((resolve) => globalThis.setTimeout(resolve, 0));

function fakeSink(frames: Frame[]) {
  const starts: number[] = [];
  const sink = {
    canvases(from = 0) {
      starts.push(from);
      const pending = frames.filter((frame) => frame.timestamp + frame.duration > from);
      return (async function* generate() {
        for (const frame of pending) {
          yield {
            canvas: frame.id as unknown as HTMLCanvasElement,
            timestamp: frame.timestamp,
            duration: frame.duration,
          };
        }
      })();
    },
  };
  return { starts, sink: sink as unknown as CanvasSink };
}

function feedOf(frames: Frame[]) {
  const { starts, sink } = fakeSink(frames);
  const decoded: string[] = [];
  const feed = createFrameFeed(sink, {
    onDecoded: () => decoded.push("decoded"),
    onFailed: () => decoded.push("failed"),
  });
  return { feed, starts, decoded };
}

/** Three presentations with a turn of the event loop between them, as the render loop does. */
async function present(frame: ReturnType<typeof feedOf>, atS: number, playing = true) {
  frame.feed.frameAt(atS, playing);
  await flush();
  frame.feed.frameAt(atS, playing);
  await flush();
  return frame.feed.frameAt(atS, playing);
}

const steady: Frame[] = [
  { id: "a", timestamp: 0, duration: 0.5 },
  { id: "b", timestamp: 0.5, duration: 0.5 },
  { id: "c", timestamp: 1, duration: 0.5 },
];

describe("video frame feed", () => {
  it("holds the last picture while a frame is still decoding", async () => {
    const frame = feedOf(steady);
    expect(frame.feed.frameAt(0, false)).toBeUndefined();
    expect(await present(frame, 0, false)).toBe("a");
    expect(frame.decoded).toContain("decoded");
  });

  it("advances to the newest frame at or before the position", async () => {
    const frame = feedOf(steady);
    await present(frame, 0);
    expect(await present(frame, 0.6)).toBe("b");
    expect(await present(frame, 1.2)).toBe("c");
  });

  it("re-seeks on a backwards scrub instead of presenting a stale frame", async () => {
    const frame = feedOf(steady);
    await present(frame, 1.2);
    expect(await present(frame, 0.1, false)).toBe("a");
    expect(frame.starts).toEqual([1.2, 0.1]);
  });

  it("seeks over a long jump rather than decoding everything in between", async () => {
    const frame = feedOf([...steady, { id: "far", timestamp: 9, duration: 0.5 }]);
    await present(frame, 0);
    await present(frame, 9);
    expect(frame.starts).toEqual([0, 9]);
  });

  it("stops requesting frames once the window covers a paused position", async () => {
    const frame = feedOf(steady);
    await present(frame, 0.1, false);
    const requested = frame.starts.length;
    frame.feed.frameAt(0.1, false);
    frame.feed.frameAt(0.1, false);
    expect(frame.starts).toHaveLength(requested);
    expect(frame.decoded.filter((entry) => entry === "decoded")).toHaveLength(1);
  });

  it("reports a decoder failure once and keeps the surface untouched", async () => {
    const broken = {
      canvases: () => ({
        next: () => Promise.reject(new Error("decoder lost")),
        return: () => Promise.resolve({ done: true, value: undefined }),
      }),
    } as unknown as CanvasSink;
    const failures: unknown[] = [];
    const feed = createFrameFeed(broken, {
      onDecoded: () => undefined,
      onFailed: (error) => failures.push(error),
    });
    feed.frameAt(0, true);
    await flush();
    feed.frameAt(0, true);
    await flush();
    expect(failures).toHaveLength(1);
    expect(feed.frameAt(0, true)).toBeUndefined();
  });

  it("presents the initial frame even when stream packets start with a positive offset", async () => {
    const offsetStream: Frame[] = [
      { id: "first", timestamp: 0.08, duration: 0.04 },
      { id: "second", timestamp: 0.12, duration: 0.04 },
    ];
    const frame = feedOf(offsetStream);
    expect(await present(frame, 0, false)).toBe("first");
    expect(frame.starts).toEqual([0]);
  });
});
