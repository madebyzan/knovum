import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { defineComponent } from "vue";

import type { GenerationPreset } from "../../../../foundation/ai/schema/models";
import { settingsLeaveCoordinatorKey } from "../../../../shared/contracts/settings-leave";
import SavedGenerationPresets from "../components/SavedGenerationPresets.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
const Input = defineComponent({ emits: ["update:modelValue"], setup: () => () => null });

it("emits a rename intent without mutating the authoritative preset", () => {
  const presets: GenerationPreset[] = [
    {
      id: "preset",
      name: "Original",
      actionId: "image-generation",
      target: { type: "agent", runtimeKind: "codex" },
      parameters: {},
    },
  ];
  const wrapper = mount(SavedGenerationPresets, {
    props: { presets, loadGenerationActions: async () => [], savePreset: async () => {} },
    global: {
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {}, requestLeave: async () => {} },
      },
      stubs: { UiInput: Input, UiIconButton: true, UiSettingsSection: { template: "<div><slot /></div>" } },
    },
  });
  wrapper.findComponent(Input).vm.$emit("update:modelValue", "Latest");
  expect(presets[0]!.name).toBe("Original");
  expect(wrapper.emitted("rename")).toEqual([["preset", "Latest"]]);
  wrapper.unmount();
});
