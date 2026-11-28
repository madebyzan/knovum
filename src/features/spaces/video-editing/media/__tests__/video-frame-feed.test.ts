import { describe, expect, it, vi } from "vitest";

import { createFrameFeed } from "../video-frames";

function fakeVideoElement() {
  const listeners = new Map<string, Set<() => void>>();
  const state = {
    src: "",
    muted: false,
    playsInline: false,
    preload: "",
    paused: true,
    seeking: false,
    readyState: 4,
    duration: 30,
    currentTime: 0,
    playbackRate: 1,
    error: null as { message: string } | null,
    playCalls: 0,
    pauseCalls: 0,
  };
  const element = {
    get src() {
      return state.src;
    },
    set src(value: string) {
      state.src = value;
    },
    get muted() {
      return state.muted;
    },
    set muted(value: boolean) {
      state.muted = value;
    },
    get playsInline() {
      return state.playsInline;
    },
    set playsInline(value: boolean) {
      state.playsInline = value;
    },
    get preload() {
      return state.preload;
    },
    set preload(value: string) {
      state.preload = value;
    },
    get paused() {
      return state.paused;
    },
    get seeking() {
      return state.seeking;
    },
    get readyState() {
      return state.readyState;
    },
    get duration() {
      return state.duration;
    },
    get currentTime() {
      return state.currentTime;
    },
    set currentTime(value: number) {
      state.currentTime = value;
    },
    get playbackRate() {
      return state.playbackRate;
    },
    set playbackRate(value: number) {
      state.playbackRate = value;
    },
    get error() {
      return state.error;
    },
    play: vi.fn<() => Promise<void>>(() => {
      state.paused = false;
      state.playCalls += 1;
      return Promise.resolve();
    }),
    pause: vi.fn<() => void>(() => {
      state.paused = true;
      state.pauseCalls += 1;
    }),
    removeAttribute: vi.fn<(name: string) => void>(() => {
      state.src = "";
    }),
    load: vi.fn<() => void>(),
    addEventListener(type: string, fn: () => void) {
      const set = listeners.get(type) ?? new Set();
      set.add(fn);
      listeners.set(type, set);
    },
    removeEventListener(type: string, fn: () => void) {
      listeners.get(type)?.delete(fn);
    },
  };
  const emit = (type: string) => {
    for (const fn of listeners.get(type) ?? []) fn();
  };
  return { element: element as unknown as HTMLVideoElement, state, emit };
}

describe("hardware video frame feed", () => {
  it("plays natively on the GPU element and smoothly phase-locks small audio-clock drift", () => {
    const { element, state } = fakeVideoElement();
    const feed = createFrameFeed(
      "asset://clip.mp4",
      { onDecoded: vi.fn<() => void>(), onFailed: vi.fn<(e: unknown) => void>() },
      () => element,
    );

    expect(feed.frameAt(1.0, true)).toBe(element);
    expect(state.paused).toBe(false);
    expect(state.currentTime).toBe(1.0);
    expect(state.playbackRate).toBe(1);

    // Small drift (80ms behind clock) nudges playbackRate instead of seeking.
    state.currentTime = 2.0;
    feed.frameAt(2.08, true);
    expect(state.playbackRate).toBe(1.04);
    expect(state.currentTime).toBe(2.0);

    // Within tight lock (<60ms) restores nominal 1.0x rate.
    state.currentTime = 2.1;
    feed.frameAt(2.11, true);
    expect(state.playbackRate).toBe(1);

    // Large cut/jump (>500ms) after startup cooldown snaps cleanly to target time.
    vi.spyOn(performance, "now").mockReturnValueOnce(performance.now() + 1_000);
    feed.frameAt(8.5, true);
    expect(state.currentTime).toBe(8.5);
    expect(state.playbackRate).toBe(1);
    feed.dispose();
  });

  it("coalesces rapid paused scrubs while a hardware seek is in flight", () => {
    const { element, state, emit } = fakeVideoElement();
    const onDecoded = vi.fn<() => void>();
    const feed = createFrameFeed(
      "asset://clip.mp4",
      { onDecoded, onFailed: vi.fn<(e: unknown) => void>() },
      () => element,
    );

    feed.frameAt(1.0, false);
    expect(state.currentTime).toBe(1.0);

    state.seeking = true;
    feed.frameAt(2.0, false);
    feed.frameAt(4.5, false);
    expect(state.currentTime).toBe(1.0);

    state.seeking = false;
    emit("seeked");
    expect(state.currentTime).toBe(4.5);
    expect(onDecoded).toHaveBeenCalled();
    feed.dispose();
  });

  it("reports media errors once and stops returning the failed element", () => {
    const { element, state, emit } = fakeVideoElement();
    const onFailed = vi.fn<(error: unknown) => void>();
    const feed = createFrameFeed(
      "asset://broken.mp4",
      { onDecoded: vi.fn<() => void>(), onFailed },
      () => element,
    );

    state.error = { message: "MEDIA_ERR_DECODE" };
    emit("error");
    emit("error");
    expect(onFailed).toHaveBeenCalledTimes(1);
    expect(feed.frameAt(0, true)).toBeUndefined();
    feed.dispose();
  });
});
