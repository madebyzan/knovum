import { describe, expect, it, vi } from "vitest";

import { afterNextPaint } from "../editor/use-deferred-editor-mount";

describe("afterNextPaint", () => {
  it("leaves one browser paint between feedback and editor mounting", async () => {
    const frames: FrameRequestCallback[] = [];
    const schedule = vi.fn<(callback: FrameRequestCallback) => number>((callback) => {
      frames.push(callback);
      return frames.length;
    });
    let resolved = false;
    const painted = afterNextPaint(schedule).then(() => (resolved = true));

    frames.shift()?.(0);
    await Promise.resolve();
    expect(resolved).toBe(false);

    frames.shift()?.(16);
    await painted;
    expect(resolved).toBe(true);
    expect(schedule).toHaveBeenCalledTimes(2);
  });
});
