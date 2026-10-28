import { mount } from "@vue/test-utils";
import { beforeEach, expect, it, vi } from "vitest";
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
});

function render() {
  return mount(SpaceTranscriptAlignmentControl, {
    props: { disabled: false, canAlign: true, selectedCount: 2 },
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
