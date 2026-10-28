import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { afterEach, expect, it } from "vitest";
import { defineComponent, h } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type { SelectedSection } from "../canvas/presentation/selection/selection-product-state";
import SpaceCanvasSectionLayoutControls from "../canvas/presentation/selection/SpaceCanvasSectionLayoutControls.vue";

let unmount: (() => void) | undefined;
afterEach(() => unmount?.());

function controls(overrides: Partial<SelectedSection>) {
  const events: { layout: unknown[]; resize: unknown[] } = { layout: [], resize: [] };
  const section: SelectedSection = {
    id: "s",
    collapsed: false,
    autoLayout: false,
    aspectLocked: false,
    appearance: {},
    frame: { x: 0, y: 0, width: 400, height: 200 },
    ...overrides,
  };
  const Host = defineComponent(
    () => () =>
      h(ToolbarRoot, null, () =>
        h(SpaceCanvasSectionLayoutControls, {
          section,
          disabled: false,
          compact: false,
          onLayout: (update: unknown) => events.layout.push(update),
          onResize: (size: unknown) => events.resize.push(size),
        }),
      ),
  );
  const wrapper = mount(Host, { global: { plugins: [i18n] }, attachTo: document.body });
  unmount = () => wrapper.unmount();
  return { wrapper, events };
}

it("reads the aspect lock from the Section and asks to store a change to it", async () => {
  const { wrapper, events } = controls({ aspectLocked: true });
  const lock = wrapper.get(
    `button[aria-label="${i18n.global.t("spaces.canvas.sections.size.unlockRatio")}"]`,
  );
  expect(lock.attributes("aria-pressed")).toBe("true");
  await lock.trigger("click");
  expect(events.layout).toEqual([{ aspectLocked: false }]);
});

it("carries the other dimension through a locked edit and commits it once", async () => {
  const { wrapper, events } = controls({ aspectLocked: true });
  const width = wrapper.findAll("input")[0]!;
  await width.setValue("800");
  await width.trigger("keydown", { key: "Enter" });
  await width.trigger("blur");
  await flushPromises();
  expect(events.resize).toEqual([{ width: 800, height: 400 }]);
});
