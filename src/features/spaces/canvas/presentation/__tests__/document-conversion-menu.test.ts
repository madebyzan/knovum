import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { expect, it, onTestFinished } from "vitest";
import { defineComponent, h, shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { localeMessages } from "../../../../../foundation/i18n/locales/locale-registry";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import { creationIntentCatalog } from "../../../knowledge-derivations/catalog/creation-intent-catalog";
import type { CreationIntentDecisions } from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceDocumentConversionMenu from "../../../knowledge-derivations/presentation/SpaceDocumentConversionMenu.vue";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpaceCanvasNodeTools from "../selection/SpaceCanvasNodeTools.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";

const node: SpaceCanvasNode = {
  id: "document",
  type: "markdown",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "markdown", title: "Report", text: "Body", knowledgeRevision: "revision-1" },
};
const actions = {
  state: shallowRef({ canOpen: false }),
  selectedVideoNode: shallowRef(undefined),
} as unknown as ReturnType<typeof useCanvasSelectionActions>;

function mountTools(source: SpaceCanvasNode, minimal = false) {
  const wrapper = mount(
    defineComponent({
      setup: () => () =>
        h(ToolbarRoot, {}, () =>
          h(SpaceCanvasNodeTools, {
            actions,
            node: source,
            derivations: creationIntentCatalog([source]).decisions,
            minimal,
            separating: false,
            separationBusy: false,
            separationUnavailableReason: undefined,
          }),
        ),
    }),
    {
      attachTo: document.body,
      global: {
        plugins: [createI18n({ legacy: false, locale: "en", messages: localeMessages })],
        stubs: { SpaceNodeSpeechActions: true },
      },
    },
  );
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("offers supported Markdown formats directly and emits the existing derivation action", async () => {
  const wrapper = mountTools(node);
  await wrapper.get('[data-testid="document-conversion-trigger"]').trigger("keydown", { key: "ArrowDown" });
  await flushPromises();
  expect(
    Array.from(document.querySelectorAll("[data-conversion-action]")).map((item) =>
      item.getAttribute("data-conversion-action"),
    ),
  ).toEqual([
    "convert-document-to-docx",
    "convert-document-to-pdf",
    "convert-document-to-txt",
    "convert-document-to-html",
    "render-presentation",
  ]);
  document.querySelector<HTMLElement>('[data-conversion-action="convert-document-to-pdf"]')!.click();
  await flushPromises();
  expect(wrapper.getComponent(SpaceCanvasNodeTools).emitted("invokeDerivation")).toEqual([
    ["convert-document-to-pdf"],
  ]);
});

it("does not offer conversion for an unindexed document or the minimal toolbar", () => {
  const unindexed = { ...node, data: { kind: "markdown" as const, title: "Report", text: "Body" } };
  expect(mountTools(unindexed).find('[data-testid="document-conversion-trigger"]').exists()).toBe(false);
  expect(mountTools(node, true).find('[data-testid="document-conversion-trigger"]').exists()).toBe(false);
});

it("keeps compact conversion labelled and disables actions blocked by a node operation", async () => {
  const derivations: CreationIntentDecisions = creationIntentCatalog([node]).decisions;
  derivations["convert-document-to-pdf"] = { state: "blocked", reason: "active-node-operation" };
  const wrapper = mount(
    defineComponent({
      setup: () => () =>
        h(ToolbarRoot, {}, () => h(SpaceDocumentConversionMenu, { decisions: derivations, isCompact: true })),
    }),
    {
      attachTo: document.body,
      global: { plugins: [createI18n({ legacy: false, locale: "en", messages: localeMessages })] },
    },
  );
  onTestFinished(() => wrapper.unmount());
  const trigger = wrapper.get('[data-testid="document-conversion-trigger"]');
  expect(trigger.attributes("aria-label")).toBe("Convert to");
  await trigger.trigger("keydown", { key: "ArrowDown" });
  await flushPromises();
  const blocked = document.querySelector<HTMLElement>('[data-conversion-action="convert-document-to-pdf"]')!;
  expect(blocked.getAttribute("aria-disabled")).toBe("true");
  blocked.click();
  await flushPromises();
  expect(wrapper.getComponent(SpaceDocumentConversionMenu).emitted("invoke")).toBeUndefined();
});

it("keeps the same conversion actions reachable from the overflow submenu", async () => {
  const wrapper = mount(
    defineComponent({
      setup: () => () =>
        h(
          UiDropdownMenu,
          {},
          {
            trigger: () => h("button", { "data-testid": "more" }, "More"),
            default: () =>
              h(SpaceDocumentConversionMenu, {
                decisions: creationIntentCatalog([node]).decisions,
                isSubmenu: true,
              }),
          },
        ),
    }),
    {
      attachTo: document.body,
      global: { plugins: [createI18n({ legacy: false, locale: "en", messages: localeMessages })] },
    },
  );
  onTestFinished(() => wrapper.unmount());
  await wrapper.get('[data-testid="more"]').trigger("keydown", { key: "ArrowDown" });
  await flushPromises();
  document
    .querySelector<HTMLElement>('[data-testid="document-conversion-trigger"]')!
    .dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
  await flushPromises();
  const menu = wrapper.getComponent(SpaceDocumentConversionMenu);
  document.querySelector<HTMLElement>('[data-conversion-action="convert-document-to-html"]')!.click();
  await flushPromises();
  expect(menu.emitted("invoke")).toEqual([["convert-document-to-html"]]);
});
