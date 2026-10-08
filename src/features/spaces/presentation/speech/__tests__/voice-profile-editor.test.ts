import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";
import { createI18n } from "vue-i18n";

import { localeMessages } from "../../../../../locales/locale-registry";
import type { VoiceProfileHistory } from "../../../contracts/generated/VoiceProfileHistory";
import VoiceProfileEditor from "../VoiceProfileEditor.vue";

it.each([
  ["en", 0, "Used in 0 segments"],
  ["en", 1, "Used in 1 segment"],
  ["en", 2, "Used in 2 segments"],
  ["zh-CN", 0, "已用于 0 个片段"],
  ["zh-CN", 1, "已用于 1 个片段"],
  ["zh-CN", 2, "已用于 2 个片段"],
] as const)("renders the complete voice reference count in %s at %i", (locale, count, expected) => {
  const profile: VoiceProfileHistory = {
    id: "voice",
    spaceId: "space",
    name: "Narrator",
    sourceKind: "preset",
    activeRevisionId: "revision",
    archivedAt: null,
    updatedAt: 1,
    referencedSegmentCount: count,
    revisions: [],
  };
  const i18n = createI18n({ legacy: false, locale, fallbackLocale: false, messages: localeMessages });
  const wrapper = mount(VoiceProfileEditor, {
    props: { profile, targets: [], disabled: false },
    global: { plugins: [i18n], stubs: { VoiceRevisionHistory: true } },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.text()).toContain(expected);
});
