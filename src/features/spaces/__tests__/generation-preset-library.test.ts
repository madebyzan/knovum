import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { expect, it, vi } from "vitest";

import type { GenerationPreset } from "../../../foundation/ai/schema/models";
import GenerationPresetLibrary from "../presentation/derivations/GenerationPresetLibrary.vue";

const calls = vi.hoisted(() => ({
  save: vi.fn<(library: unknown) => Promise<void>>(),
  error: vi.fn<(message: string) => void>(),
  success: vi.fn<(message: string) => void>(),
}));
vi.mock("../../../foundation/ai/integration-settings", () => ({
  loadCreationLibrary: async () => ({ generationPresets: [] }),
  saveCreationLibrary: calls.save,
  resetCreationLibrary: async () => {},
}));
vi.mock("vue-sonner", () => ({ toast: { error: calls.error, success: calls.success } }));

it("presents a failed preset save without reporting success", async () => {
  calls.save.mockRejectedValue(new Error("Disk unavailable"));
  const preset: GenerationPreset = {
    id: "preset",
    name: "Preset",
    actionId: "image-generation",
    target: { type: "agent", runtimeKind: "codex" },
    parameters: {},
  };
  const wrapper = mount(GenerationPresetLibrary, {
    props: {
      actionId: preset.actionId,
      createPreset: () => preset,
      label: "Presets",
      emptyLabel: "Empty",
      placeholder: "Select",
      saveLabel: "Save",
      savedSuccessMessage: "Saved",
      saveDisabled: false,
    },
    global: {
      plugins: [createPinia()],
      stubs: { UiSelect: true, UiButton: { template: "<button><slot /></button>" } },
    },
  });
  await flushPromises();
  await wrapper.find("button").trigger("click");
  await flushPromises();
  expect(calls.error).toHaveBeenCalledWith("Disk unavailable");
  expect(calls.success).not.toHaveBeenCalled();
  wrapper.unmount();
});
