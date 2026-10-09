import type { MenuItemOptions } from "@tauri-apps/api/menu";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import type * as Reka from "reka-ui";
import { beforeEach, expect, it, vi } from "vitest";
import { h, reactive, shallowRef } from "vue";

import { i18n } from "../../../foundation/i18n/i18n";
import { workspaceSidebarKey } from "../../../shared/contracts/workspace-sidebar";
import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import SpaceOutline from "../presentation/outline/SpaceOutline.vue";

const native = vi.hoisted(() => ({ items: [] as MenuItemOptions[] }));
vi.mock("@tauri-apps/api/menu", () => ({
  Menu: { new: async () => ({ popup: async () => {}, close: async () => {} }) },
  MenuItem: {
    new: async (options: MenuItemOptions) => {
      native.items.push(options);
      return { rid: native.items.length, close: async () => {} };
    },
  },
  PredefinedMenuItem: { new: async () => ({ rid: 0, close: async () => {} }) },
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => store }));
vi.mock("../canvas/editor-session", () => ({ getCanvasEditorSession: () => editor }));
vi.mock("../canvas/use-node-operation-gate", () => ({
  useNodeOperationGate: () => ({ allows: () => true }),
}));
vi.mock("reka-ui", async (importOriginal) => ({
  ...(await importOriginal<typeof Reka>()),
  TreeVirtualizer: {
    setup(_: unknown, { slots }: { slots: Record<string, (value: unknown) => unknown> }) {
      const virtualizer = { scrollToIndex: () => {}, resizeItem: () => {} };
      return () =>
        h(
          "div",
          slots.default!({
            item: {
              level: 1,
              value: {
                id: "content",
                kind: "content",
                locked: store.editSnapshot.structure[0]!.locked,
                title: store.nodes[0]!.data.title,
              },
            },
            virtualizer,
            virtualItem: { index: 0 },
          }) as ReturnType<typeof h>[],
        );
    },
  },
}));

const editor = {
  selection: shallowRef({ nodeIds: [], structureNodeIds: [] }),
  selectNodes: vi.fn<(ids: readonly string[]) => void>(),
  selectStructureNodes: vi.fn<(ids: readonly string[]) => void>(),
  reveal: vi.fn<(ids: readonly string[], mode: "focus" | "ensure-visible") => void>(),
};
const store = reactive({
  activeSpaceId: "space",
  outlineRenameStructureNodeId: null,
  groupScopePath: [] as string[],
  nodes: [{ id: "image", data: { kind: "image", title: "Original" } }],
  editSnapshot: {
    editRevision: 0,
    structure: [{ id: "content", parentId: null, order: 0, kind: "content", nodeId: "image", locked: false }],
    layouts: [],
    relationStyles: [],
  } as SpaceEditSnapshot,
  renameNode: vi.fn<(nodeId: string, title: string) => Promise<void>>(async (_nodeId, title) => {
    store.nodes[0]!.data.title = title;
  }),
  executeSpaceEdit: vi.fn<(command: SpaceEditCommand) => Promise<void>>(async (command) => {
    if (command.kind === "structure.set-lock")
      store.editSnapshot.structure[0]!.locked = Boolean(command.locked);
  }),
});

beforeEach(() => {
  vi.clearAllMocks();
  native.items.length = 0;
  store.nodes[0]!.data.title = "Original";
  store.editSnapshot.structure[0]!.locked = false;
});

async function renderMenu() {
  const wrapper = mount(SpaceOutline, {
    attachTo: document.body,
    global: {
      plugins: [createPinia(), i18n],
      provide: { [workspaceSidebarKey as symbol]: { reveal: () => {} } },
    },
  });
  await wrapper.get('[role="treeitem"]').trigger("contextmenu");
  await flushPromises();
  return wrapper;
}

it("focuses the canvas object from its native locate action", async () => {
  const wrapper = await renderMenu();
  try {
    native.items[0]!.action!("locate");
    await flushPromises();
    expect(editor.selectNodes).toHaveBeenCalledWith(["image"]);
    expect(editor.reveal).toHaveBeenCalledExactlyOnceWith(["image"], "focus");
  } finally {
    wrapper.unmount();
  }
});

it("opens the inline rename field and persists the name from its native action", async () => {
  const wrapper = await renderMenu();
  try {
    native.items[1]!.action!("rename");
    await flushPromises();
    expect(document.activeElement).toBe(wrapper.get("input").element);
    await wrapper.get("input").setValue("Renamed");
    await wrapper.get("input").trigger("keydown", { key: "Enter" });
    await flushPromises();
    expect(store.renameNode).toHaveBeenCalledExactlyOnceWith("image", "Renamed");
    expect(wrapper.find("input").exists()).toBe(false);
    expect(wrapper.text()).toContain("Renamed");
  } finally {
    wrapper.unmount();
  }
});

it("locks and unlocks the object from successive native menus", async () => {
  const wrapper = await renderMenu();
  try {
    native.items[2]!.action!("lock");
    await flushPromises();
    expect(store.executeSpaceEdit).toHaveBeenLastCalledWith({
      kind: "structure.set-lock",
      rootIds: ["content"],
      locked: true,
    });
    expect(wrapper.find(".lucide-lock-keyhole").exists()).toBe(true);
    await wrapper.get('[role="treeitem"]').trigger("contextmenu");
    await flushPromises();
    native.items[5]!.action!("unlock");
    await flushPromises();
    expect(store.executeSpaceEdit).toHaveBeenLastCalledWith({
      kind: "structure.set-lock",
      rootIds: ["content"],
      locked: false,
    });
    expect(wrapper.find(".lucide-lock-keyhole").exists()).toBe(false);
  } finally {
    wrapper.unmount();
  }
});
