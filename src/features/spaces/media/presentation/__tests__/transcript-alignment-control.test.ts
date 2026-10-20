import { mount } from "@vue/test-utils";
import { beforeEach, expect, it, vi } from "vitest";
import { shallowRef, type ShallowRef } from "vue";

import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import SpaceTranscriptAlignmentControl from "../SpaceTranscriptAlignmentControl.vue";

type ModelState = {
  data: ShallowRef<LocalModelAvailability[] | undefined>;
  isPending: ShallowRef<boolean>;
  error: ShallowRef<Error | null>;
};
const models = vi.hoisted(() => ({ useLocalModelsQuery: vi.fn<() => ModelState>() }));
vi.mock("../../../../../foundation/runtime/use-local-models-query", () => models);
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
const selected: LocalModelAvailability = {
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
  downloadAvailable: true,
  unavailableReason: null,
};
beforeEach(() => {
  models.useLocalModelsQuery.mockReturnValue({
    data: shallowRef([selected]),
    isPending: shallowRef(false),
    error: shallowRef(null),
  });
});

it("emits align event when the alignment button is clicked", async () => {
  const wrapper = mount(SpaceTranscriptAlignmentControl, {
    props: { disabled: false, canAlign: true, selectedCount: 2 },
  });
  await wrapper.get('[data-testid="transcript-align"]').trigger("click");
  expect(wrapper.emitted("align")).toHaveLength(1);
});

it("shows a model-query failure and prevents a request with unknown resource state", () => {
  models.useLocalModelsQuery.mockReturnValue({
    data: shallowRef(undefined),
    isPending: shallowRef(false),
    error: shallowRef(new Error("Model catalog unavailable")),
  });
  const wrapper = mount(SpaceTranscriptAlignmentControl, {
    props: { disabled: false, canAlign: true, selectedCount: 1 },
  });
  expect(wrapper.get('[role="alert"]').text()).toBe("Model catalog unavailable");
  expect(wrapper.get('[data-testid="transcript-align"]').attributes("disabled")).toBeDefined();
});
