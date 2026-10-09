import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import SpaceVideoPlayer from "../presentation/media/SpaceVideoPlayer.vue";

vi.mock("../media/media-element-source", () => ({
  versionedMediaSource: (path: string | undefined) => path,
  waitForMediaMetadata: vi
    .fn<(element: HTMLMediaElement, message: string) => Promise<void>>()
    .mockRejectedValue(new Error("Metadata failed")),
}));

const data: SpaceVideoPayload = {
  kind: "video",
  title: "Interview",
  assetPath: "/spaces/interview.mp4",
  fileName: "interview.mp4",
};
const wrappers: ReturnType<typeof mount>[] = [];
beforeEach(() => {
  vi.stubGlobal(
    "HTMLMediaElement",
    class extends HTMLMediaElement {
      static override readonly HAVE_METADATA = 1;
    },
  );
});
afterEach(() => {
  for (const wrapper of wrappers) wrapper.unmount();
  wrappers.length = 0;
  vi.unstubAllGlobals();
});

it("rejects imperative playback failures so the calling operation can handle them", async () => {
  const wrapper = mount(SpaceVideoPlayer, { props: { data }, global: { plugins: [i18n] } });
  wrappers.push(wrapper);
  await expect(wrapper.vm.togglePlayback()).rejects.toThrow("Metadata failed");
  expect(wrapper.emitted("playbackError")).toBeUndefined();
});

it("reports playback failures from the player's button as a component event", async () => {
  const wrapper = mount(SpaceVideoPlayer, { props: { data }, global: { plugins: [i18n] } });
  wrappers.push(wrapper);
  await wrapper.get("button").trigger("click");
  await vi.waitFor(() => expect(wrapper.emitted("playbackError")).toEqual([["Metadata failed"]]));
});
