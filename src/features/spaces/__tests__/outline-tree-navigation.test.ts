import { mount } from "@vue/test-utils";
import { TreeRoot, type FlattenedItem } from "reka-ui";
import { afterEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, nextTick, shallowRef } from "vue";

import { isInteractiveTarget } from "../../../foundation/dom/interactive-target";
import { i18n } from "../../../foundation/i18n/i18n";
import type { OutlineTreeItem } from "../presentation/outline/outline-tree-items";
import SpaceStructureTreeItem from "../presentation/outline/SpaceStructureTreeItem.vue";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));

const items: OutlineTreeItem[] = [
  {
    children: [
      {
        children: [{ id: "note", kind: "content", locked: false, title: "Note" }],
        id: "section",
        kind: "section",
        locked: false,
        title: "Section",
      },
      { id: "leaf", kind: "content", locked: false, title: "Leaf" },
    ],
    id: "group",
    kind: "group",
    locked: false,
    title: "Group",
  },
];

function createHarness(initiallyExpanded: string[] = []) {
  const expanded = shallowRef<string[]>(initiallyExpanded);
  const selectedIds: string[] = [];

  function toggleId(id: string) {
    expanded.value = expanded.value.includes(id)
      ? expanded.value.filter((value) => value !== id)
      : [...expanded.value, id];
  }

  const Harness = defineComponent({
    setup: () => () =>
      h(
        TreeRoot,
        {
          asChild: true,
          expanded: expanded.value,
          getKey: (item: Record<string, unknown>) => String(item.id),
          items,
          "onUpdate:expanded": (value: string[]) => {
            expanded.value = value;
          },
        },
        {
          default: ({ flattenItems }: { flattenItems: FlattenedItem<OutlineTreeItem>[] }) => [
            h(
              "section",
              { "aria-label": "Outline" },
              flattenItems.map(({ level, value }) =>
                h(SpaceStructureTreeItem, {
                  editing: false,
                  expanded: expanded.value.includes(value.id),
                  item: value,
                  key: value.id,
                  level,
                  saveName: async () => undefined,
                  selected: false,
                  onSelect: () => selectedIds.push(value.id),
                  onToggle: () => toggleId(value.id),
                }),
              ),
            ),
          ],
        },
      ),
  });

  return { expanded, Harness, selectedIds };
}

function mountHarness(initiallyExpanded?: string[]) {
  const harness = createHarness(initiallyExpanded);
  const wrapper = mount(harness.Harness, {
    attachTo: document.body,
    global: { plugins: [i18n] },
  });
  return { ...harness, wrapper };
}

function tabbableRows() {
  return [...document.querySelectorAll("[role='treeitem'][tabindex='0']")].map((row) =>
    row.textContent?.trim(),
  );
}

describe("outline tree navigation", () => {
  let wrapper: ReturnType<typeof mount> | undefined;

  afterEach(() => {
    wrapper?.unmount();
    document.body.innerHTML = "";
  });

  it("levels and indent rows by depth and marks only parents expandable", () => {
    ({ wrapper } = mountHarness(["group", "section"]));
    const rows = wrapper.findAll("[role='treeitem']");

    expect(rows.map((row) => row.attributes("aria-level"))).toEqual(["1", "2", "3", "2"]);
    expect(rows.map((row) => row.attributes("aria-expanded"))).toEqual([
      "true",
      "true",
      undefined,
      undefined,
    ]);
    expect(rows.map((row) => row.element.parentElement?.parentElement?.style.paddingLeft)).toEqual([
      "0px",
      "16px",
      "32px",
      "16px",
    ]);
  });

  it("routes keyboard entry through the tree container and roves focus between rows", async () => {
    ({ wrapper } = mountHarness(["group"]));
    await nextTick();

    expect(wrapper.get("section").attributes("tabindex")).toBe("0");
    expect(tabbableRows()).toEqual([]);

    await wrapper.findAll("[role='treeitem']")[2]!.trigger("focus");
    expect(tabbableRows()).toEqual(["Leaf"]);
  });

  it("keeps rows buttons so bare keys are suppressed while the outline has focus", () => {
    ({ wrapper } = mountHarness(["group"]));

    for (const row of wrapper.findAll("[role='treeitem']")) {
      expect(isInteractiveTarget(row.element)).toBe(true);
    }
  });

  it("selects on a row click without disclosing the row", async () => {
    const harness = mountHarness();
    wrapper = harness.wrapper;

    await wrapper.get("[role='treeitem']").trigger("click");

    expect(harness.selectedIds).toEqual(["group"]);
    expect(harness.expanded.value).toEqual([]);
    expect(wrapper.findAll("[role='treeitem']")).toHaveLength(1);
  });

  it("discloses from the chevron and from ArrowRight", async () => {
    const harness = mountHarness();
    wrapper = harness.wrapper;

    await wrapper.get("[role='treeitem']").trigger("keydown", { key: "ArrowRight" });
    expect(harness.expanded.value).toEqual(["group"]);
    expect(wrapper.findAll("[role='treeitem']")).toHaveLength(3);

    await wrapper.get("button:not([role='treeitem'])").trigger("click");
    await nextTick();
    expect(harness.expanded.value).toEqual([]);
    expect(wrapper.findAll("[role='treeitem']")).toHaveLength(1);
  });
});
