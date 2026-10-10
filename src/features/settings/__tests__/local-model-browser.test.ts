import { mount, type VueWrapper } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import UiSelect from "../../../shared/ui/UiSelect.vue";
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
  essential: false,
  selected: false,
  selectable: false,
  fixed: false,
  totalBytes: 100,
  installedBytes: installed ? 100 : 0,
  downloadAvailable: true,
  unavailableReason: null,
}));
vi.mock("../../../foundation/runtime/use-resource-queries", () => ({
  useLocalModelsQuery: () => ({
    data: shallowRef(catalog),
    error: shallowRef(),
    isPending: shallowRef(false),
  }),
}));
function render() {
  const wrapper = mount(LocalModelsSettings, {
    attachTo: document.body,
    global: {
      plugins: [i18n],
      stubs: {
        LocalModelConfiguration: defineComponent({
          props: ["model"],
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
async function selectCapability(wrapper: VueWrapper, capability: string) {
  wrapper.getComponent(UiSelect).vm.$emit("update:modelValue", capability);
  await wrapper.vm.$nextTick();
}
it("switches capability without stacking all model groups in a long settings page", async () => {
  const wrapper = render();
  await selectCapability(wrapper, "transcription");
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="qwen3Embedding"]').isVisible()).toBe(false);
  expect(wrapper.get('[data-model-id="mossNano"]').isVisible()).toBe(false);
});
it("filters the active capability by model name and installed state", async () => {
  const wrapper = render();
  await selectCapability(wrapper, "transcription");
  await wrapper.get("input").setValue("tiny");
  expect(wrapper.get('[data-model-id="whisperTiny"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(false);
  await wrapper.get("input").setValue("");
  wrapper.findAllComponents(UiSelect)[1]!.vm.$emit("update:modelValue", "installed");
  await wrapper.vm.$nextTick();
  expect(wrapper.get('[data-model-id="whisperSmall"]').isVisible()).toBe(true);
  expect(wrapper.get('[data-model-id="whisperTiny"]').isVisible()).toBe(false);
  await wrapper.get("input").setValue("nonexistent");
  expect(
    wrapper.get('[data-model-capability="transcription"] [data-testid="model-no-results"]').isVisible(),
  ).toBe(true);
});
it("retains model action state through filtering and capability switching", async () => {
  const wrapper = render();
  await selectCapability(wrapper, "transcription");
  await wrapper.get('[data-model-id="whisperSmall"]').trigger("click");
  await wrapper.get("input").setValue("tiny");
  await selectCapability(wrapper, "speechSynthesis");
  await selectCapability(wrapper, "transcription");
  expect((wrapper.get("input").element as HTMLInputElement).value).toBe("");
  expect(wrapper.get('[data-model-id="whisperSmall"]').text()).toBe("whisperSmall 1");
});
