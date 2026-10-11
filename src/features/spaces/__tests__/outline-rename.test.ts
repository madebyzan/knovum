import { flushPromises, mount } from "@vue/test-utils";
import { TreeRoot } from "reka-ui";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import SpaceStructureTreeItem from "../presentation/outline/SpaceStructureTreeItem.vue";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));

function render(saveName: (title: string) => Promise<unknown>) {
  const item = { id: "note", kind: "content" as const, locked: false, title: "Original" };
  const wrapper = mount(
    defineComponent({
      setup: () => () =>
        h(
          TreeRoot,
          { items: [item], getKey: (value) => String(value.id) },
          {
            default: () =>
              h(SpaceStructureTreeItem, {
                item,
                level: 1,
                editing: true,
                expanded: false,
                selected: true,
                saveName,
              }),
          },
        ),
    }),
    { attachTo: document.body, global: { plugins: [i18n] } },
  );
  onTestFinished(() => wrapper.unmount());
  return { wrapper, row: wrapper.getComponent(SpaceStructureTreeItem) };
}

it("keeps the rename draft active if the field loses focus during a pending save", async () => {
  let rejectSave!: (reason: Error) => void;
  const save = vi.fn<(title: string) => Promise<unknown>>().mockReturnValue(
    new Promise((_resolve, reject) => {
      rejectSave = reject;
    }),
  );
  const { wrapper, row } = render(save);
  await flushPromises();
  await wrapper.get("input").setValue("Changed");
  await wrapper.get("input").trigger("keydown", { key: "Enter" });
  await wrapper.get("input").trigger("blur");
  expect(row.emitted("finish")).toBeUndefined();
  rejectSave(new Error("Write failed"));
  await flushPromises();
  expect(wrapper.get("input").element.value).toBe("Changed");
  expect(wrapper.find('[role="alert"]').exists()).toBe(true);
  expect(document.activeElement).toBe(wrapper.get("input").element);
});

it("commits once, trims the title and ignores IME confirmation", async () => {
  const save = vi.fn<(title: string) => Promise<unknown>>().mockResolvedValue(undefined);
  const { wrapper, row } = render(save);
  await flushPromises();
  await wrapper.get("input").setValue("  Changed  ");
  await wrapper.get("input").trigger("keydown", { key: "Enter", isComposing: true });
  expect(save).not.toHaveBeenCalled();
  await wrapper.get("input").trigger("keydown", { key: "Enter" });
  await flushPromises();
  expect(save).toHaveBeenCalledExactlyOnceWith("Changed");
  expect(row.emitted("finish")).toHaveLength(1);
});
