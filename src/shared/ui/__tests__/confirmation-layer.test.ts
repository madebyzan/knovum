import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";
import { defineComponent, shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { confirmAction, resolveConfirmation } from "../confirm-action";
import UiConfirmAction from "../UiConfirmAction.vue";
import UiDialog from "../UiDialog.vue";

const i18n = createI18n({
  legacy: false,
  locale: "en",
  messages: { en: { common: { cancel: "Cancel", close: "Close" } } },
});

const ACTION = { title: "Discard", description: "Discard the draft?", confirmLabel: "Discard" };

/**
 * Portalled content is inserted at the teleport anchor its component created on mount, so the
 * confirmation host mounted at application start sits earlier in the document than any dialog
 * opened later. Only a higher stacking band keeps the confirmation reachable.
 */
const Host = defineComponent({
  components: { UiDialog, UiConfirmAction },
  setup() {
    const dialogMounted = shallowRef(false);
    const open = shallowRef(false);
    return { dialogMounted, open };
  },
  template: `
    <div>
      <UiConfirmAction />
      <UiDialog v-if="dialogMounted" :open="open" title="Editor" />
    </div>
  `,
});

function settle() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function stackingBand(surface: HTMLElement) {
  const band = /(?:^|\s)z-(\d+)(?:\s|$)/.exec(surface.className)?.[1];
  expect(band).toBeDefined();
  return Number(band);
}

it("raises a confirmation above the dialog that requested it", async () => {
  const wrapper = mount(Host, { attachTo: document.body, global: { plugins: [i18n] } });
  onTestFinished(() => {
    resolveConfirmation(false);
    wrapper.unmount();
  });
  wrapper.vm.dialogMounted = true;
  await settle();
  wrapper.vm.open = true;
  await settle();
  const pending = confirmAction(ACTION);
  await settle();

  const surfaces = [...document.querySelectorAll<HTMLElement>("[role='dialog']")];
  const confirmation = surfaces.find((surface) => surface.textContent?.includes(ACTION.description));
  const dialog = surfaces.find((surface) => surface !== confirmation);
  expect(confirmation).toBeDefined();
  expect(dialog).toBeDefined();
  // The confirmation is painted first in document order, so it must outrank the dialog by band.
  expect(surfaces.indexOf(confirmation!)).toBeLessThan(surfaces.indexOf(dialog!));
  expect(stackingBand(confirmation!)).toBeGreaterThan(stackingBand(dialog!));

  resolveConfirmation(true);
  await expect(pending).resolves.toBe(true);
});
