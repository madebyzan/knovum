import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import type { AiReadiness } from "../../../../../foundation/ai/ai-readiness";
import type * as LocalModelChoice from "../../../../../foundation/ai/local-model-choice";
import { localModelAvailability } from "../../../__tests__/support/image-operation-fixtures";
import SpaceDiarizationControl from "../SpaceDiarizationControl.vue";

const mocks = vi.hoisted(() => ({
  readiness: undefined as unknown as { value: AiReadiness },
  admit: vi.fn<() => Promise<boolean>>(),
  request: vi.fn<() => Promise<void>>(),
  refresh: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../../../../foundation/ai/use-ai-readiness", () => ({ useAiReadiness: () => mocks.readiness }));
vi.mock("../../../../../foundation/ai/ai-readiness-admission", () => ({ admitAiRequirements: mocks.admit }));
vi.mock("../../../../../foundation/ai/local-model-choice", async (importOriginal) => ({
  ...(await importOriginal<typeof LocalModelChoice>()),
  preferredLocalModel: async () => "sherpaDiarization" as const,
}));
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
  mockIPC((command) => {
    if (command !== "plugin:local-resources|get_model_catalog")
      throw new Error(`Unexpected command: ${command}`);
    return [];
  });
});
afterEach(clearMocks);
function render(props: { speakerCount?: number } = {}) {
  const wrapper = mount(SpaceDiarizationControl, {
    props: { nodeId: "node", revision: 2, durationMs: 10_000, speakerCount: 0, disabled: false, ...props },
    global: {
      plugins: [
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
    },
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
    model: "sherpaDiarization",
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
  const wrapper = render({ speakerCount: 3 });
  expect(wrapper.text()).toContain("3");
});

it("opens a one-click popover when multiple models are downloaded and submits the chosen one", async () => {
  mockIPC((command) => {
    if (command !== "plugin:local-resources|get_model_catalog")
      throw new Error(`Unexpected command: ${command}`);
    return [
      localModelAvailability("sherpaDiarization", "speechDiarization", {
        selected: true,
        verified: true,
        group: "audio",
      }),
      localModelAvailability("sherpaDiarizationFull", "speechDiarization", {
        verified: true,
        group: "audio",
      }),
    ];
  });
  const wrapper = mount(SpaceDiarizationControl, {
    attachTo: document.body,
    props: { nodeId: "node-1", revision: 3, durationMs: 60_000, speakerCount: 0, disabled: false },
    global: {
      plugins: [
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
    },
  });
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(mocks.request).not.toHaveBeenCalled();
  const option = Array.from(document.body.querySelectorAll("button")).find((button) =>
    button.textContent?.includes("sherpaDiarizationFull"),
  );
  expect(option).toBeDefined();
  option!.click();
  await flushPromises();
  expect(mocks.request).toHaveBeenCalledWith(expect.objectContaining({ model: "sherpaDiarizationFull" }));
});
