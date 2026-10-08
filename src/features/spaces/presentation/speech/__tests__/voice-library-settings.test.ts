import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { reactive } from "vue";
import type * as VueI18n from "vue-i18n";

import SpaceVoiceLibrarySettings from "../SpaceVoiceLibrarySettings.vue";

vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => store }));
vi.mock("../../../speech/ipc", () => ({
  createPresetVoice: vi.fn<() => Promise<void>>(),
  listVoiceRevisions: vi.fn<() => Promise<[]>>(),
}));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));
const store = reactive({
  activeSpaceId: "first",
  spaces: [
    { id: "first", title: "First", archivedAt: null },
    { id: "second", title: "Second", archivedAt: null },
    { id: "archived", title: "Archived", archivedAt: 1 },
  ],
});

it("manages a chosen Space without switching the canvas session or mixing reference owners", async () => {
  const wrapper = mount(SpaceVoiceLibrarySettings, {
    global: {
      stubs: {
        UiSelect: {
          props: ["modelValue", "options"],
          emits: ["update:modelValue"],
          template:
            '<select :value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"><option v-for="option in options" :key="option.value" :value="option.value">{{ option.label }}</option></select>',
        },
        VoiceCandidateForm: {
          props: ["spaceId"],
          template: '<div data-testid="references" :data-space-id="spaceId" />',
        },
        VoiceProfileForm: {
          props: ["spaceId"],
          template: '<div data-testid="presets" :data-space-id="spaceId" />',
        },
        VoiceManagement: {
          props: ["spaceId"],
          template: '<div data-testid="management" :data-space-id="spaceId" />',
        },
      },
    },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.findAll("option")).toHaveLength(2);
  await wrapper.get("select").setValue("second");
  await flushPromises();
  for (const testId of ["references", "presets", "management"]) {
    expect(wrapper.get(`[data-testid="${testId}"]`).attributes("data-space-id")).toBe("second");
  }
  expect(store.activeSpaceId).toBe("first");
});
