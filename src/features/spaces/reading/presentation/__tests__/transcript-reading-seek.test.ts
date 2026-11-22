import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import { transcriptFixture } from "../../../__tests__/support/media-transcript-fixture";
import { createSpaceNode } from "../../../canvas/node";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../../../model/nodes/node-payload";
import SpaceTranscriptReadingSurface from "../SpaceTranscriptReadingSurface.vue";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://localhost${path}`,
}));

enableAutoUnmount(afterEach);

function videoNode(): SpaceCanvasNode<SpaceVideoPayload> {
  return createSpaceNode(
    {
      kind: "video",
      title: "Interview",
      fileName: "interview.mp4",
      assetPath: "/interview.mp4",
      sourceRevision: "rev-1",
      mediaKnowledge: {
        sourceRevision: "rev-1",
        schemaVersion: 1,
        transcript: transcriptFixture("rev-1", [
          { startMs: 0, endMs: 5_000, text: "Opening" },
          { startMs: 12_500, endMs: 16_000, text: "I auditioned when I was nine" },
        ]),
      },
    },
    { x: 0, y: 0 },
    "video-1",
  ) as SpaceCanvasNode<SpaceVideoPayload>;
}

describe("SpaceTranscriptReadingSurface media citation playback", () => {
  it("waits for video metadata on initial open, then seeks and plays in the reading area", async () => {
    const playSpy = vi.spyOn(HTMLMediaElement.prototype, "play").mockResolvedValue(undefined);
    const wrapper = mount(SpaceTranscriptReadingSurface, {
      props: {
        node: videoNode(),
        locator: { sourceRevision: "rev-1", location: { kind: "media", startMs: 12_500, endMs: 16_000 } },
        locateNonce: 1,
        quote: "I auditioned when I was nine",
      },
      global: { plugins: [i18n], stubs: { SpaceNodeTranscriptionAction: true } },
    });

    const video = wrapper.get("video").element as HTMLVideoElement;
    Object.defineProperty(video, "duration", { configurable: true, value: 120 });
    Object.defineProperty(video, "readyState", { configurable: true, value: HTMLMediaElement.HAVE_METADATA });
    video.dispatchEvent(new Event("loadedmetadata"));
    await flushPromises();

    expect(video.currentTime).toBe(12.5);
    expect(playSpy).toHaveBeenCalledTimes(1);

    // Clicking the same citation again with a new nonce re-seeks and plays even after scrubbing away.
    video.currentTime = 40;
    await wrapper.setProps({ locateNonce: 2 });
    await flushPromises();

    expect(video.currentTime).toBe(12.5);
    expect(playSpy).toHaveBeenCalledTimes(2);
    playSpy.mockRestore();
  });
});
