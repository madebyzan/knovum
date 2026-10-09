import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/i18n";
import { overlayScopeVisibleKey } from "../../../../../shared/contracts/overlay-scope-visibility";
import SpeechMediaPlayer from "../SpeechMediaPlayer.vue";

it("uses Knovum controls and stops a reference audition at the requested interval end", async () => {
  const wrapper = mount(SpeechMediaPlayer, {
    props: { kind: "video", source: "/source.mp4", preview: false },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  const media = wrapper.get<HTMLVideoElement>("video").element;
  Object.defineProperties(media, { duration: { value: 60 }, readyState: { value: 1 } });
  vi.spyOn(media, "play").mockImplementation(async () => {
    media.dispatchEvent(new Event("play"));
  });
  const pause = vi.spyOn(media, "pause").mockImplementation(() => {
    media.dispatchEvent(new Event("pause"));
  });
  await wrapper.get("video").trigger("loadedmetadata");
  expect(wrapper.emitted("loaded")).toEqual([[60_000]]);
  expect(wrapper.find("[controls]").exists()).toBe(false);
  expect(wrapper.get("video").classes()).toContain("hidden");
  await wrapper.vm.seekTo(5000, 10000);
  await flushPromises();
  expect(media.currentTime).toBe(5);
  expect(wrapper.emitted("play")).toHaveLength(1);
  media.currentTime = 10;
  await wrapper.get("video").trigger("timeupdate");
  await flushPromises();
  expect(pause).toHaveBeenCalled();
});

it("pauses when its overlay scope hides and releases playback on unmount", async () => {
  const visible = shallowRef(true);
  const wrapper = mount(SpeechMediaPlayer, {
    props: { kind: "audio", source: "/source.wav" },
    global: { plugins: [i18n], provide: { [overlayScopeVisibleKey as symbol]: visible } },
  });
  const media = wrapper.get<HTMLAudioElement>("audio").element;
  const pause = vi.spyOn(media, "pause").mockImplementation(() => media.dispatchEvent(new Event("pause")));
  await wrapper.get("audio").trigger("play");
  visible.value = false;
  await flushPromises();
  expect(pause).toHaveBeenCalled();
  pause.mockClear();
  wrapper.unmount();
  expect(pause).toHaveBeenCalledOnce();
});
