import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";

import SpaceDiarizationControl from "../presentation/media/SpaceDiarizationControl.vue";

const mocks = vi.hoisted(() => ({
  data: {
    value: [
      {
        capability: "speechDiarization",
        selected: true,
        verified: false,
        name: "INT8",
        totalBytes: 41_146_685,
      },
    ],
  },
  error: { value: null },
  request: vi.fn<() => Promise<void>>(),
  refresh: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../../foundation/runtime/use-resource-queries", () => ({ useLocalModelsQuery: () => mocks }));
vi.mock("../media/transcription", () => ({ requestMediaDiarization: mocks.request }));
vi.mock("../enrichment/enrichment-jobs-query", () => ({ refreshEnrichmentJobs: mocks.refresh }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.data.value = [
    {
      capability: "speechDiarization",
      selected: true,
      verified: false,
      name: "INT8",
      totalBytes: 41_146_685,
    },
  ];
});
function render() {
  const wrapper = mount(SpaceDiarizationControl, {
    props: { nodeId: "node", revision: 2, durationMs: 10_000, speakerCount: 0, disabled: false },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("starts speaker recognition on the selected node without an account or a manually prepared model", async () => {
  const wrapper = render();
  const button = wrapper.get("button");
  expect(button.attributes("disabled")).toBeUndefined();
  await button.trigger("click");
  await flushPromises();
  expect(mocks.request).toHaveBeenCalledWith({
    spaceId: "space",
    nodeId: "node",
    expectedRevision: 2,
    numSpeakers: null,
  });
  expect(mocks.refresh).toHaveBeenCalledOnce();
});

it("passes an optional known speaker count and rejects invalid counts", async () => {
  const wrapper = render();
  await wrapper.get("input").setValue("4");
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(mocks.request).toHaveBeenCalledWith(expect.objectContaining({ numSpeakers: 4 }));
  await wrapper.get("input").setValue("33");
  expect(wrapper.get("button").attributes("disabled")).toBeDefined();
});

it.each([false, true])("shows the selected variant and its own readiness: verified=%s", (verified) => {
  mocks.data.value = [
    {
      capability: "speechDiarization",
      selected: false,
      verified: true,
      name: "INT8",
      totalBytes: 41_146_685,
    },
    { capability: "speechDiarization", selected: true, verified, name: "FP32", totalBytes: 45_599_092 },
  ];
  const wrapper = render();
  expect(wrapper.get('[data-testid="speaker-model"]').text()).toContain("FP32");
  expect(wrapper.find('[data-testid="speaker-first-use"]').exists()).toBe(!verified);
});
