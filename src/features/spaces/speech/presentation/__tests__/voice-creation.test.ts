import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { defineComponent } from "vue";
import type * as VueI18n from "vue-i18n";

import VoiceCreation from "../VoiceCreation.vue";

vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

const VoiceCandidateForm = defineComponent({ template: '<div data-testid="candidate-form" />' });
const VoiceProfileForm = defineComponent({ template: '<div data-testid="profile-form" />' });

function render() {
  return mount(VoiceCreation, {
    props: { kind: "preset", creating: false, errors: [] },
    global: { stubs: { VoiceCandidateForm, VoiceProfileForm } },
  });
}

// Leaving a tab can discard a draft form behind an asynchronous confirmation, so a switch request
// must come from an explicit activation only: focus alone (roving focus, or the focus a closing
// confirmation restores) must not re-arm the dialog the user just answered.
it("requests a tab switch only from an explicit activation", async () => {
  const wrapper = render();
  const tabs = wrapper.findAll('[role="tab"]');
  await tabs[1]!.trigger("focus");
  expect(wrapper.emitted("select")).toBeUndefined();
  await tabs[1]!.trigger("mousedown", { button: 0 });
  expect(wrapper.emitted("select")).toEqual([["reference"]]);
});
