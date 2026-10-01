import { flushPromises, shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { reactive } from "vue";

import type { GenerationPreset } from "../../../foundation/ai/schema/models";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import GenerationPresetEditor from "../components/GenerationPresetEditor.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

describe("generation preset editor", () => {
  it("opens, edits and saves a reactive preset without mutating the source", async () => {
    const source = reactive<GenerationPreset>({
      id: "preset",
      name: "Original",
      actionId: "generate_image",
      target: { type: "agent", runtimeKind: "codex" },
      parameters: { width: 1024 },
    });
    const wrapper = shallowMount(GenerationPresetEditor, {
      props: {
        open: true,
        preset: source,
        loadGenerationActions: async () => [{ id: "generate_image", routingPurpose: "image-generation" }],
      },
      global: { stubs: { UiDialog: { template: "<div><slot /><slot name='footer' /></div>" } } },
    });
    await flushPromises();
    wrapper.findComponent(UiInput).vm.$emit("update:modelValue", "  Edited  ");
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");
    const saved = wrapper.emitted("save")![0]![0];
    expect(saved).toEqual({ ...source, name: "Edited" });
    expect(() => structuredClone(saved)).not.toThrow();
    expect(source.name).toBe("Original");
    await wrapper.setProps({ open: false });
    await wrapper.setProps({ open: true });
    expect(wrapper.findComponent(UiInput).props("modelValue")).toBe("Original");
    wrapper.unmount();
  });
});
