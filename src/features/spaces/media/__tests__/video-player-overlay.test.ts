import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";

import { useVideoPlayerOverlay } from "../use-video-player-overlay";

describe("useVideoPlayerOverlay", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("remains visible when video is paused", () => {
    const playing = ref(false);
    const scope = effectScope();
    const overlay = scope.run(() => useVideoPlayerOverlay(playing))!;

    expect(overlay.controlsVisible.value).toBe(true);
    vi.advanceTimersByTime(3000);
    expect(overlay.controlsVisible.value).toBe(true);

    scope.stop();
  });

  it("auto-hides controls after inactivity while playing", () => {
    const playing = ref(true);
    const scope = effectScope();
    const overlay = scope.run(() => useVideoPlayerOverlay(playing))!;

    overlay.wakeControls();
    expect(overlay.controlsVisible.value).toBe(true);

    vi.advanceTimersByTime(2000);
    expect(overlay.controlsVisible.value).toBe(false);

    overlay.wakeControls();
    expect(overlay.controlsVisible.value).toBe(true);

    scope.stop();
  });

  it("hides immediately on pointer leave if video is playing", () => {
    const playing = ref(true);
    const scope = effectScope();
    const overlay = scope.run(() => useVideoPlayerOverlay(playing))!;

    overlay.wakeControls();
    expect(overlay.controlsVisible.value).toBe(true);

    overlay.handlePointerLeave();
    expect(overlay.controlsVisible.value).toBe(false);

    scope.stop();
  });

  it("keeps controls visible on pointer leave if video is paused", () => {
    const playing = ref(false);
    const scope = effectScope();
    const overlay = scope.run(() => useVideoPlayerOverlay(playing))!;

    overlay.handlePointerLeave();
    expect(overlay.controlsVisible.value).toBe(true);

    scope.stop();
  });

  it("triggers and automatically clears flash action feedback", () => {
    const playing = ref(false);
    const scope = effectScope();
    const overlay = scope.run(() => useVideoPlayerOverlay(playing))!;

    overlay.triggerFlash("play");
    expect(overlay.flashFeedback.value).toBe("play");

    vi.advanceTimersByTime(450);
    expect(overlay.flashFeedback.value).toBeUndefined();

    scope.stop();
  });
});
