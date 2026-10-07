import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";

import SpaceDiarizationControl from "../presentation/media/SpaceDiarizationControl.vue";

const mocks = vi.hoisted(() => ({
  data: { value: { models: { diarization: { verified: false, tokenConfigured: true } } } },
  error: { value: null },
  request: vi.fn<() => Promise<void>>(),
  refresh: vi.fn<() => Promise<void>>(),
  push: vi.fn<(route: { path: string; query: { pane: string } }) => Promise<void>>(),
}));
vi.mock("../../../foundation/runtime/resource-query", () => ({ useLocalResourcesQuery: () => mocks }));
vi.mock("../media/transcription", () => ({ requestMediaDiarization: mocks.request }));
vi.mock("../enrichment/enrichment-jobs-query", () => ({ refreshEnrichmentJobs: mocks.refresh }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
vi.mock("vue-router", () => ({ useRouter: () => ({ push: mocks.push }) }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.data.value.models.diarization = { verified: false, tokenConfigured: true };
});
function render() {
  const wrapper = mount(SpaceDiarizationControl, {
    props: { nodeId: "node", revision: 2, durationMs: 10_000, speakerCount: 0, disabled: false },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("starts speaker recognition on the selected node before the authorized model is installed", async () => {
  const wrapper = render();
  const button = wrapper.get("button");
  expect(button.attributes("disabled")).toBeUndefined();
  await button.trigger("click");
  await flushPromises();
  expect(mocks.request).toHaveBeenCalledWith({
    spaceId: "space",
    nodeId: "node",
    expectedRevision: 2,
    minimumSpeakers: null,
    maximumSpeakers: null,
  });
  expect(mocks.refresh).toHaveBeenCalledOnce();
});

it("provides a direct access-configuration action when official authorization is missing", async () => {
  mocks.data.value.models.diarization.tokenConfigured = false;
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  expect(mocks.push).toHaveBeenCalledWith({ path: "/settings", query: { pane: "aiModels" } });
  expect(mocks.request).not.toHaveBeenCalled();
});
