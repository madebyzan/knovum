import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import SpeechProjectActions from "../SpeechProjectActions.vue";

function render() {
  const wrapper = mount(SpeechProjectActions, {
    attachTo: document.body,
    props: {
      canGenerate: true,
      projectType: "short_dubbing",
      isPreviewing: false,
      isGenerating: false,
      isSubmitting: false,
      isSaving: false,
      isCancelling: false,
      isPreviewListening: false,
    },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("keeps generation as the primary action and preserves focus while saving", async () => {
  const wrapper = render();
  const primary = wrapper.get<HTMLButtonElement>('[data-testid="speech-generate"]').element;
  primary.focus();
  await wrapper.get('[data-testid="speech-generate"]').trigger("click");
  expect(wrapper.emitted("generate")).toHaveLength(1);
  await wrapper.setProps({ isSaving: true });
  expect(wrapper.get('[data-testid="speech-generate"]').attributes("disabled")).toBeDefined();
  await wrapper.setProps({ isSaving: false });
  expect(wrapper.get('[data-testid="speech-generate"]').element).toBe(primary);
  expect(document.activeElement).toBe(primary);
  expect(wrapper.findAll("button")).toHaveLength(1);
});

it("offers no generation while the Task Center is carrying a Run for this project", async () => {
  const wrapper = render();
  await wrapper.setProps({ isGenerating: true });
  expect(wrapper.find('[data-testid="speech-cancel"]').exists()).toBe(false);
  expect(wrapper.get('[data-testid="speech-generate"]').attributes("disabled")).toBeDefined();
});

it("cancels the audition it is running and locks the control during cancellation", async () => {
  const wrapper = render();
  await wrapper.setProps({ isPreviewing: true });
  expect(wrapper.findAll("button")).toHaveLength(1);
  await wrapper.get('[data-testid="speech-cancel"]').trigger("click");
  expect(wrapper.emitted("cancel")).toHaveLength(1);
  await wrapper.setProps({ isCancelling: true });
  expect(wrapper.get('[data-testid="speech-cancel"]').attributes("disabled")).toBeDefined();
});
