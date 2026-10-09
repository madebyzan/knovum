import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { reactive, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import { workspaceSidebarKey } from "../../../shared/contracts/workspace-sidebar";
import SpaceOutline from "../presentation/outline/SpaceOutline.vue";

vi.mock("../session/space-store", () => ({ useSpaceStore: () => store }));
vi.mock("../canvas/editor-session", () => ({
  getCanvasEditorSession: () => ({
    selection: shallowRef({ nodeIds: [], structureNodeIds: [] }),
  }),
}));
vi.mock("../canvas/use-node-operation-gate", () => ({
  useNodeOperationGate: () => ({ allows: () => true }),
}));
vi.mock("../session/node-operation-drafts-store", () => ({
  useNodeOperationDraftsStore: () => ({
    begin: () => 1,
    end: () => {},
  }),
}));
vi.mock("../presentation/outline/use-structure-outline-menu", () => ({
  useStructureOutlineMenu: () => ({ show: () => {} }),
}));

const store = reactive({
  activeSpaceId: "space",
  outlineRenameStructureNodeId: null as string | null,
  nodes: [{ id: "note", data: { title: "Note" } }],
  editSnapshot: {
    structure: [
      { id: "section", parentId: null, order: 0, kind: "section", title: "Chapter", locked: false },
      { id: "content", parentId: "section", order: 0, kind: "content", nodeId: "note", locked: false },
    ],
  },
});

it("reveals a rename target inside a collapsed chapter", async () => {
  const reveal = vi.fn<() => void>();
  const wrapper = mount(SpaceOutline, {
    global: {
      plugins: [i18n],
      provide: { [workspaceSidebarKey as symbol]: { reveal } },
      stubs: { TreeVirtualizer: true },
    },
  });
  try {
    store.outlineRenameStructureNodeId = "content";
    await flushPromises();
    expect(wrapper.getComponent({ name: "TreeRoot" }).props("expanded")).toEqual(["section"]);
    expect(reveal).toHaveBeenCalledOnce();
  } finally {
    wrapper.unmount();
  }
});
