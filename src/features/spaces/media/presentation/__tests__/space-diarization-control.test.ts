import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import type { AiReadiness } from "../../../../../foundation/ai/ai-readiness";
import SpaceDiarizationControl from "../SpaceDiarizationControl.vue";

const mocks = vi.hoisted(() => ({
  readiness: undefined as unknown as { value: AiReadiness },
  admit: vi.fn<() => Promise<boolean>>(),
  request: vi.fn<() => Promise<void>>(),
  refresh: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../../../../foundation/ai/use-ai-readiness", () => ({ useAiReadiness: () => mocks.readiness }));
vi.mock("../../../../../foundation/ai/ai-readiness-admission", () => ({ admitAiRequirements: mocks.admit }));
vi.mock("../../transcription", () => ({ requestMediaDiarization: mocks.request }));
vi.mock("../../../enrichment/enrichment-jobs-query", () => ({ refreshEnrichmentJobs: mocks.refresh }));
vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => ({ activeSpaceId: "space" }) }));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.readiness = shallowRef<AiReadiness>({ state: "ready" });
  mocks.admit.mockResolvedValue(true);
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

it("does not request speaker recognition that its local model admission refused", async () => {
  mocks.admit.mockResolvedValue(false);
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(mocks.admit).toHaveBeenCalledWith([{ kind: "localCapability", capability: "speechDiarization" }]);
  expect(mocks.request).not.toHaveBeenCalled();
});

it("cannot start while the local model state is unknown", () => {
  mocks.readiness = shallowRef<AiReadiness>({ state: "unavailable", source: "models" });
  const wrapper = render();
  expect(wrapper.get("button").attributes("disabled")).toBeDefined();
  expect(wrapper.find('[data-testid="ai-readiness-notice"]').exists()).toBe(true);
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

it("displays speaker count when speakers are identified", () => {
  const wrapper = mount(SpaceDiarizationControl, {
    props: { nodeId: "node", revision: 2, durationMs: 10_000, speakerCount: 3, disabled: false },
  });
  expect(wrapper.text()).toContain("3");
});
