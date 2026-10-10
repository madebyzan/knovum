import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../../../foundation/i18n/i18n";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import SpaceUnsavedChangesDialog from "../SpaceUnsavedChangesDialog.vue";

it("provides a usable save action unless the caller explicitly disallows saving", async () => {
  const wrapper = mount(SpaceUnsavedChangesDialog, {
    attachTo: document.body,
    props: { open: true, saving: false, title: "Save?", description: "Save or discard." },
    global: { plugins: [i18n], stubs: { UiDialog: { template: '<div><slot name="footer" /></div>' } } },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.findAll("button")).toHaveLength(3);
  const save = wrapper.findAll("button")[2]!;
  expect(save.attributes("disabled")).toBeUndefined();
  await save.trigger("click");
  expect(wrapper.emitted("save")).toEqual([[]]);
  await wrapper.setProps({ canSave: false });
  expect(save.attributes("disabled")).toBeDefined();
});

it("omits an unavailable save action while preserving cancellation and discard", async () => {
  const wrapper = mount(SpaceUnsavedChangesDialog, {
    attachTo: document.body,
    props: {
      open: true,
      saving: false,
      showSave: false,
      title: "Discard?",
      description: "Finish or discard.",
    },
    global: { plugins: [i18n], stubs: { UiDialog: { template: '<div><slot name="footer" /></div>' } } },
  });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.findAllComponents(UiButton)).toHaveLength(2);
  await wrapper.findAllComponents(UiButton)[0]!.trigger("click");
  expect(wrapper.emitted("cancel")).toEqual([[]]);
  await wrapper.findAllComponents(UiButton)[1]!.trigger("click");
  expect(wrapper.emitted("discard")).toEqual([[]]);
  await wrapper.setProps({ showSave: true, saving: true });
  const save = wrapper.findAllComponents(UiButton)[2]!;
  expect(save.attributes("disabled")).toBeDefined();
  await wrapper.setProps({ saving: false });
  const enabledSave = wrapper.findAll("button")[2]!;
  expect(enabledSave.attributes("disabled")).toBeUndefined();
  await enabledSave.trigger("click");
  expect(wrapper.emitted("save")).toEqual([[]]);
});
