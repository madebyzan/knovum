import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";

import { i18n } from "../../../../../foundation/i18n/i18n";
import type { VideoDubbingOutputConfig } from "../../../contracts/generated/VideoDubbingOutputConfig";
import VideoSourceControls from "../VideoSourceControls.vue";

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => path }));
it("keeps original-interval playback compact while the selected canvas video remains the preview", () => {
  const config: VideoDubbingOutputConfig = {
    format: "mp4",
    video: { assetPath: "/video.mp4", sourceRevision: `sha256:${"a".repeat(64)}`, durationMs: 40000 },
    background: null,
    backgroundSeparation: null,
    voiceGainDb: 0,
    backgroundGainDb: -12,
    translation: null,
    preserveOriginalOutsideDialogue: false,
    sourceDialogueIntervals: [],
    retainOriginalTrack: true,
  };
  const wrapper = mount(VideoSourceControls, {
    props: { modelValue: config, lines: [], voiceOptions: [], disabled: false },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.get("video").classes()).toContain("hidden");
  expect(wrapper.find("[controls]").exists()).toBe(false);
});
