import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";
import { computed, defineComponent, inject, shallowRef } from "vue";

import { overlayScopeVisibleKey } from "../../../../../shared/contracts/overlay-scope-visibility";
import SpeechEditorPage from "../SpeechEditorPage.vue";
import SpeechEditorTabs from "../SpeechEditorTabs.vue";

it("retains script drafts across pages and hides inactive audition scopes", async () => {
  const Draft = defineComponent({
    setup() {
      return { text: shallowRef("Original"), visible: inject(overlayScopeVisibleKey) };
    },
    template: '<input v-model="text" :data-audition-visible="visible" />',
  });
  const Host = defineComponent({
    components: { SpeechEditorTabs, SpeechEditorPage, Draft },
    setup: () => ({ page: shallowRef("script") }),
    template: `<SpeechEditorTabs v-model="page" :items="[{ value: 'script', label: 'Script' }, { value: 'settings', label: 'Settings' }]" label="Editor">
      <SpeechEditorPage value="script" :active="page"><Draft /></SpeechEditorPage>
      <SpeechEditorPage value="settings" :active="page"><p>Settings</p></SpeechEditorPage>
    </SpeechEditorTabs>`,
  });
  const parentVisible = shallowRef(true);
  const wrapper = mount(Host, {
    attachTo: document.body,
    global: { provide: { [overlayScopeVisibleKey as symbol]: computed(() => parentVisible.value) } },
  });
  onTestFinished(() => wrapper.unmount());
  const input = wrapper.get("input");
  await input.setValue("Edited draft");
  await wrapper
    .get('[role="tab"][data-state="inactive"]')
    .trigger("mousedown", { button: 0, ctrlKey: false });
  expect(input.isVisible()).toBe(false);
  expect(input.attributes("data-audition-visible")).toBe("false");
  await wrapper
    .get('[role="tab"][data-state="inactive"]')
    .trigger("mousedown", { button: 0, ctrlKey: false });
  expect(input.isVisible()).toBe(true);
  expect(input.element.value).toBe("Edited draft");
  parentVisible.value = false;
  await wrapper.vm.$nextTick();
  expect(input.attributes("data-audition-visible")).toBe("false");
});

it("reveals the page containing invalid configuration", async () => {
  const wrapper = mount(SpeechEditorTabs, {
    props: {
      modelValue: "script",
      label: "Editor",
      items: [
        { value: "script", label: "Script" },
        { value: "settings", label: "Settings" },
      ],
    },
  });
  onTestFinished(() => wrapper.unmount());
  await wrapper.setProps({ invalidPage: "settings" });
  expect(wrapper.emitted("update:modelValue")?.at(-1)).toEqual(["settings"]);
});
