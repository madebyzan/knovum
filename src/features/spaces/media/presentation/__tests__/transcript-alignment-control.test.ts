import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";
import type * as VueI18n from "vue-i18n";

import type { AiReadiness } from "../../../../../foundation/ai/ai-readiness";
import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import SpaceTranscriptAlignmentControl from "../SpaceTranscriptAlignmentControl.vue";

const readiness = vi.hoisted(() => ({ current: undefined as unknown as { value: AiReadiness } }));
vi.mock("../../../../../foundation/ai/use-ai-readiness", () => ({ useAiReadiness: () => readiness.current }));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));
const aligner: LocalModelAvailability = {
  id: "qwen3AlignerQ8",
  name: "Qwen3 Q8",
  capability: "speechAlignment",
  operations: [],
  referenceTranscript: "unused",
  referenceDuration: null,
  group: "audio",
  essential: false,
  selected: true,
  selectable: true,
  fixed: false,
  totalBytes: 1_129_987_258,
  installedBytes: 0,
  installed: false,
  verified: false,
  memoryAdvisoryBytes: null,
};
beforeEach(() => {
  readiness.current = shallowRef<AiReadiness>({ state: "ready" });
  mockIPC((command) => {
    if (command !== "plugin:local-resources|get_model_catalog")
      throw new Error(`Unexpected command: ${command}`);
    return [];
  });
});
afterEach(clearMocks);

function render() {
  return mount(SpaceTranscriptAlignmentControl, {
    props: { disabled: false, canAlign: true, selectedCount: 2 },
    global: {
      plugins: [
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
    },
  });
}

it("emits align event when the alignment button is clicked", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="transcript-align"]').trigger("click");
  expect(wrapper.emitted("align")).toHaveLength(1);
});

it("explains the aligner download before it runs, without holding the request back", () => {
  readiness.current = shallowRef<AiReadiness>({
    state: "download",
    models: [aligner],
    bytes: aligner.totalBytes,
  });
  const wrapper = render();
  expect(wrapper.get('[data-testid="ai-readiness-notice"]').text()).toContain("Qwen3 Q8");
  expect(wrapper.get('[data-testid="transcript-align"]').attributes("disabled")).toBeUndefined();
});

it("shows a model-query failure and prevents a request with unknown resource state", () => {
  readiness.current = shallowRef<AiReadiness>({ state: "unavailable", source: "models" });
  const wrapper = render();
  expect(wrapper.get('[role="alert"]').text()).toContain("Local models couldn't be checked.");
  expect(wrapper.get('[data-testid="transcript-align"]').attributes("disabled")).toBeDefined();
});

it("opens a one-click popover when multiple aligner models are downloaded", async () => {
  mockIPC((command) => {
    if (command !== "plugin:local-resources|get_model_catalog")
      throw new Error(`Unexpected command: ${command}`);
    return [
      { ...aligner, installed: true, verified: true, selected: true },
      {
        ...aligner,
        id: "qwen3Aligner",
        name: "Qwen3 FP16",
        installed: true,
        verified: true,
        selected: false,
      },
    ];
  });
  const wrapper = mount(SpaceTranscriptAlignmentControl, {
    attachTo: document.body,
    props: { disabled: false, canAlign: true, selectedCount: 2 },
    global: {
      plugins: [
        [VueQueryPlugin, { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) }],
      ],
    },
  });
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  await wrapper.get('[data-testid="transcript-align"]').trigger("click");
  await flushPromises();
  expect(wrapper.emitted("align")).toBeUndefined();
  const option = Array.from(document.body.querySelectorAll("button")).find((button) =>
    button.textContent?.includes("Qwen3 FP16"),
  );
  expect(option).toBeDefined();
  option!.click();
  await flushPromises();
  expect(wrapper.emitted("align")?.at(-1)).toEqual(["qwen3Aligner"]);
});
