import { mount, flushPromises } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpaceNodeTranscriptionAction from "../SpaceNodeTranscriptionAction.vue";

const mocks = vi.hoisted(() => ({ analyze: vi.fn<() => Promise<void>>() }));
vi.mock("../../use-media-knowledge", () => ({
  useMediaKnowledge: () => ({ canTranscribe, transcribed, transcribing, analyzing, analyze: mocks.analyze }),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("../../../../../foundation/runtime/use-local-models-query", () => ({
  localModelsQueryOptions: { queryKey: ["local-resources", "models"] },
  useLocalModelsQuery: () => ({ data: { value: [] } }),
}));
const canTranscribe = shallowRef(true);
const transcribed = shallowRef(false);
const transcribing = shallowRef(false);
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
  transcribing.value = false;
  analyzing.value = false;
});
it("starts transcription directly from the media node toolbar", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="node-transcription-action"]').trigger("click");
  expect(mocks.analyze).toHaveBeenCalledOnce();
});
it("offers retranscription when a completed transcript already exists", async () => {
  transcribed.value = true;
  canTranscribe.value = true;
  const wrapper = render();
  await flushPromises();
  const action = wrapper.get('[data-testid="node-transcription-action"]');
  expect(action.attributes("disabled")).toBeUndefined();
  expect(action.attributes("aria-label")).toBe("spaces.media.player.retranscribe");
  await action.trigger("click");
  expect(mocks.analyze).toHaveBeenCalledOnce();
});
it("keeps the queued or running Rust task visible and prevents duplicate dispatch", async () => {
  transcribed.value = true;
  transcribing.value = true;
  analyzing.value = true;
  canTranscribe.value = false;
  const wrapper = render();
  await flushPromises();
  const action = wrapper.get('[data-testid="node-transcription-action"]');
  expect(action.attributes("disabled")).toBeDefined();
  expect(action.attributes("aria-label")).toBe("spaces.media.player.transcribing");
});
