import { mount, flushPromises } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpaceNodeTranscriptionAction from "../SpaceNodeTranscriptionAction.vue";

const mocks = vi.hoisted(() => ({ analyze: vi.fn<() => Promise<void>>() }));
vi.mock("../../../media/use-media-knowledge", () => ({
  useMediaKnowledge: () => ({ canTranscribe, transcribed, analyzing, analyze: mocks.analyze }),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
const canTranscribe = shallowRef(true);
const transcribed = shallowRef(false);
const analyzing = shallowRef(false);
const node: SpaceCanvasNode = {
  id: "audio",
  type: "audio",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "audio", title: "Audio", fileName: "audio.wav", assetPath: "/audio.wav" },
};
function render() {
  const wrapper = mount(
    defineComponent({
      setup: () => () => h(ToolbarRoot, {}, { default: () => h(SpaceNodeTranscriptionAction, { node }) }),
    }),
  );
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}
beforeEach(() => {
  vi.clearAllMocks();
  canTranscribe.value = true;
  transcribed.value = false;
  analyzing.value = false;
});
it("starts transcription directly from the media node toolbar", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="node-transcription-action"]').trigger("click");
  expect(mocks.analyze).toHaveBeenCalledOnce();
});
it("shows a completed transcript without offering destructive retranscription", async () => {
  transcribed.value = true;
  canTranscribe.value = false;
  const wrapper = render();
  await flushPromises();
  const action = wrapper.get('[data-testid="node-transcription-action"]');
  expect(action.attributes("disabled")).toBeDefined();
  expect(action.attributes("aria-label")).toBe("spaces.media.player.transcribed");
});
it("keeps the queued or running native task visible and prevents duplicate dispatch", async () => {
  analyzing.value = true;
  canTranscribe.value = false;
  const wrapper = render();
  await flushPromises();
  const action = wrapper.get('[data-testid="node-transcription-action"]');
  expect(action.attributes("disabled")).toBeDefined();
  expect(action.attributes("aria-label")).toBe("spaces.media.player.transcribing");
});
