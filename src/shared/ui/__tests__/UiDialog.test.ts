import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, expect, it } from "vitest";
import { createI18n } from "vue-i18n";

import UiDialog from "../UiDialog.vue";

const i18n = createI18n({ legacy: false, locale: "en", messages: { en: { common: { close: "Close" } } } });
let wrapper: ReturnType<typeof mount>;
afterEach(() => {
  wrapper.unmount();
  document.body.innerHTML = "";
});
async function open(description?: string) {
  wrapper = mount(UiDialog, {
    props: { open: true, title: "Review installation", description },
    global: { plugins: [i18n] },
    attachTo: document.body,
  });
  await flushPromises();
  return document.querySelector('[role="dialog"]')!;
}
it("links its accessible description to the rendered description", async () => {
  const description = "Review the included capabilities";
  const dialog = await open(description);
  const id = dialog.getAttribute("aria-describedby");
  expect(id).not.toBeNull();
  expect(document.getElementById(id!)?.textContent).toBe(description);
});
it("omits a description relationship when no description exists", async () => {
  const dialog = await open();
  expect(dialog.hasAttribute("aria-describedby")).toBe(false);
});
