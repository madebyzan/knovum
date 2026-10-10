import { mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import LocalModelCapability from "../components/LocalModelCapability.vue";
import LocalModelsSettings from "../components/LocalModelsSettings.vue";

const catalog: LocalModelAvailability[] = [
  ["qwen3Embedding", "embedding", "knowledge", true],
  ["whisperSmall", "transcription", "audio", true],
  ["whisperTiny", "transcription", "audio", false],
  ["mossNano", "speechSynthesis", "audio", false],
].map(([id, capability, group, installed]) => ({
  id: id as LocalModelAvailability["id"],
  name: String(id),
  capability: String(capability),
  group: group as LocalModelAvailability["group"],
  installed: Boolean(installed),
  verified: Boolean(installed),
  operations: [],
  referenceTranscript: "unused",
  essential: id === "qwen3Embedding",
  selected: id === "whisperSmall" || id === "mossNano",
  selectable: false,
  fixed: id === "qwen3Embedding",
  totalBytes: 100,
  installedBytes: installed ? 100 : 0,
  downloadAvailable: true,
  unavailableReason: null,
}));
const modelsQuery = {
  data: shallowRef<LocalModelAvailability[] | undefined>(catalog),
  error: shallowRef<Error>(),
  isPending: shallowRef(false),
  isFetching: shallowRef(false),
  refetch: vi.fn<() => Promise<void>>(),
};
vi.mock("../../../foundation/runtime/use-resource-queries", () => ({
  useLocalModelsQuery: () => modelsQuery,
}));
beforeEach(() => {
  modelsQuery.data.value = catalog;
  modelsQuery.error.value = undefined;
  modelsQuery.isPending.value = false;
  modelsQuery.isFetching.value = false;
  modelsQuery.refetch.mockReset();
});
function render() {
  const wrapper = mount(LocalModelsSettings, {
    attachTo: document.body,
    global: {
      plugins: [i18n],
      stubs: {
        LocalModelConfiguration: defineComponent({
          name: "LocalModelConfiguration",
          props: ["model"],
          emits: ["preparationChange", "failure"],
          setup() {
            const count = shallowRef(0);
            return { count };
          },
          template:
            '<button :data-model-id="model.id" @click="count++">{{ model.name }} {{ count }}</button>',
        }),
      },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}
it("shows every capability's configured model without opening its model controls", async () => {
  const wrapper = render();
  expect(wrapper.get('[data-model-group="knowledge"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-group="audio"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-testid="model-capability-transcription"]').text()).toContain("whisperSmall");
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(false);
  await wrapper.get('[data-testid="model-capability-transcription"]').trigger("click");
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="qwen3Embedding"]').isVisible()).toBe(false);
});
it("finds models across capabilities and filters them by installed state", async () => {
  const wrapper = render();
  await wrapper.get("input").setValue("tiny");
  expect(wrapper.get('[data-model-id="whisperTiny"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(false);
  await wrapper.get("input").setValue("moss");
  expect(wrapper.get('[data-model-id="mossNano"]').isVisible()).toBe(true);
  await wrapper.get("input").setValue("");
  wrapper.getComponent(UiSelect).vm.$emit("update:modelValue", "installed");
  await wrapper.vm.$nextTick();
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="qwen3Embedding"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperTiny"]').isVisible()).toBe(false);
  await wrapper.get("input").setValue("nonexistent");
  expect(wrapper.get('[data-testid="model-no-results"]').isVisible()).toBe(true);
  expect(wrapper.findAll('[data-testid="model-no-results"]')).toHaveLength(1);
});
it("finds capabilities by their user-facing name", async () => {
  const wrapper = render();
  await wrapper.get("input").setValue(i18n.global.t("settings.ai.localModels.categoryLabels.transcription"));
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperTiny"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="mossNano"]').isVisible()).toBe(false);
});
it("restores disclosure state after a search and retains the model controls", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="model-capability-transcription"]').trigger("click");
  await wrapper.get('[data-model-id="whisperSmall"]').trigger("click");
  await wrapper.get('[data-testid="model-capability-transcription"]').trigger("click");
  await wrapper.get('[data-testid="model-capability-transcription"]').trigger("click");
  expect(wrapper.get('[data-model-id="whisperSmall"]').text()).toBe("whisperSmall 1");
  await wrapper.get("input").setValue("moss");
  expect(wrapper.get('[data-model-id="mossNano"]').isVisible()).toBe(true);
  await wrapper.get("input").setValue("");
  expect(wrapper.get('[data-model-id="mossNano"]').isVisible()).toBe(false);
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperSmall"]').text()).toBe("whisperSmall 1");
});
it("keeps active preparation visible in a collapsed capability", async () => {
  const wrapper = render();
  const capability = wrapper
    .findAllComponents(LocalModelCapability)
    .find((item) => item.props("capability") === "transcription")!;
  const controls = capability.findAllComponents({ name: "LocalModelConfiguration" });
  controls[0]!.vm.$emit("preparationChange", true);
  await wrapper.vm.$nextTick();
  expect(capability.get('[role="status"]').isVisible()).toBe(true);
  controls[0]!.vm.$emit("preparationChange", false);
  await wrapper.vm.$nextTick();
  expect(capability.find('[role="status"]').exists()).toBe(false);
});

it("retains expanded controls and their state after a background refresh fails", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="model-capability-transcription"]').trigger("click");
  await wrapper.get('[data-model-id="whisperSmall"]').trigger("click");
  modelsQuery.error.value = new Error("Catalog refresh failed");
  await wrapper.vm.$nextTick();
  expect(wrapper.get('[role="alert"]').text()).toContain("Catalog refresh failed");
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperSmall"]').text()).toBe("whisperSmall 1");
  await wrapper.get('[role="alert"] button').trigger("click");
  expect(modelsQuery.refetch).toHaveBeenCalledOnce();
  modelsQuery.isFetching.value = true;
  await wrapper.vm.$nextTick();
  expect(wrapper.get('[role="alert"] button').attributes("disabled")).toBeDefined();
});
it("clears both filters from empty results and returns to keyboard search", async () => {
  const wrapper = render();
  await wrapper.get("input").setValue("nonexistent");
  wrapper.getComponent(UiSelect).vm.$emit("update:modelValue", "installed");
  await wrapper.vm.$nextTick();
  await wrapper.get('[data-testid="model-clear-filters"]').trigger("click");
  expect(wrapper.find('[data-testid="model-no-results"]').exists()).toBe(false);
  expect(wrapper.get('[data-testid="model-status-filter"]').text()).toContain(
    i18n.global.t("settings.ai.localModels.filters.all"),
  );
  expect((wrapper.get("input").element as HTMLInputElement).value).toBe("");
  expect(document.activeElement).toBe(wrapper.get("input").element);
});
it("reveals an operation failure even when its capability was collapsed", async () => {
  const wrapper = render();
  const capability = wrapper
    .findAllComponents(LocalModelCapability)
    .find((item) => item.props("capability") === "transcription")!;
  capability.getComponent({ name: "LocalModelConfiguration" }).vm.$emit("failure");
  await wrapper.vm.$nextTick();
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
});
