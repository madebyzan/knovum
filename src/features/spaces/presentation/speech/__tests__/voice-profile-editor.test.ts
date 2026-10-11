import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";
import { createI18n } from "vue-i18n";

import { localeMessages } from "../../../../../locales/locale-registry";
import UiInput from "../../../../../shared/ui/UiInput.vue";
import UiNumberField from "../../../../../shared/ui/UiNumberField.vue";
import type { VoiceProfileHistory } from "../../../contracts/generated/VoiceProfileHistory";
import VoiceProfileConfiguration from "../VoiceProfileConfiguration.vue";
import VoiceProfileEditor from "../VoiceProfileEditor.vue";

const firstRevision = {
  id: "revision-1",
  revision: 1,
  engineId: "moss_nano",
  modelSpecId: "moss-tts-nano-100m-onnx-native-v2",
  language: "zh-CN",
  speed: 1,
  sampleRate: 48_000,
  referenceSourceRevision: null,
  designDescription: "Warm narrator",
  createdAt: 1,
};
const profile: VoiceProfileHistory = {
  id: "voice",
  name: "Narrator",
  sourceKind: "reference",
  activeRevisionId: firstRevision.id,
  archivedAt: null,
  updatedAt: 1,
  referencedBindingCount: 0,
  revisions: [firstRevision],
};

function renderEditor() {
  const wrapper = mount(VoiceProfileEditor, {
    props: { profile, disabled: false },
    global: {
      plugins: [createI18n({ legacy: false, locale: "en", fallbackLocale: false, messages: localeMessages })],
      stubs: { VoiceRevisionHistory: true },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("keeps revision fields when a saved profile name arrives", async () => {
  const wrapper = renderEditor();
  wrapper.getComponent({ name: "UiSelect" }).vm.$emit("update:modelValue", "en");
  wrapper.getComponent(UiNumberField).vm.$emit("update:modelValue", 1.25);
  await wrapper.setProps({ profile: { ...profile, name: "Saved narrator", updatedAt: 2 } });
  expect(wrapper.getComponent(UiInput).props("modelValue")).toBe("Saved narrator");
  expect(wrapper.getComponent({ name: "UiSelect" }).props("modelValue")).toBe("en");
  expect(wrapper.getComponent(UiNumberField).props("modelValue")).toBe(1.25);
});

it("keeps the name draft when a saved voice revision arrives", async () => {
  const wrapper = renderEditor();
  wrapper.getComponent(UiInput).vm.$emit("update:modelValue", "Draft narrator");
  const nextRevision = { ...firstRevision, id: "revision-2", revision: 2, language: "en", speed: 1.2 };
  await wrapper.setProps({
    profile: {
      ...profile,
      activeRevisionId: nextRevision.id,
      updatedAt: 2,
      revisions: [nextRevision, firstRevision],
    },
  });
  expect(wrapper.getComponent(UiInput).props("modelValue")).toBe("Draft narrator");
  expect(wrapper.getComponent({ name: "UiSelect" }).props("modelValue")).toBe("en");
  expect(wrapper.getComponent(UiNumberField).props("modelValue")).toBe(1.2);
});

it.each([
  ["en", 0, "Referenced 0 times"],
  ["en", 1, "Referenced 1 time"],
  ["en", 2, "Referenced 2 times"],
  ["zh-CN", 0, "被引用 0 次"],
  ["zh-CN", 1, "被引用 1 次"],
  ["zh-CN", 2, "被引用 2 次"],
] as const)("renders the complete voice reference count in %s at %i", (locale, count, expected) => {
  const wrapper = mount(VoiceProfileConfiguration, {
    props: { profile: { ...profile, referencedBindingCount: count }, expanded: false, disabled: false },
    global: {
      plugins: [createI18n({ legacy: false, locale, fallbackLocale: false, messages: localeMessages })],
      stubs: { VoiceProfileEditor: true },
    },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.text()).toContain(expected);
});
