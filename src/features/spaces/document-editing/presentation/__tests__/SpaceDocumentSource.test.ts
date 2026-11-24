import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { describe, expect, it, onTestFinished, vi } from "vitest";
import { h } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { workspaceRightSidebarKey } from "../../../../../shared/contracts/workspace-right-sidebar";
import { createSpaceNode } from "../../../canvas/node";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SourceReadingTarget } from "../../../reading/source-reader-target";
import { createSpaceNotePayload } from "../../tiptap-content";
import SpaceDocumentSource from "../SpaceDocumentSource.vue";

const mocks = vi.hoisted(() => ({
  store: {
    activeSpaceId: "space-1",
    nodes: [] as SpaceCanvasNode[],
    readingTarget: undefined as SourceReadingTarget | undefined,
    readingHistory: [] as readonly unknown[],
    readingHistoryIndex: -1,
    readingFollowsSelection: false,
    readerFocusMode: false,
    closeSourceReader: vi.fn<() => void>(),
  },
}));

vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => mocks.store }));
vi.mock("../../../reading/presentation/SpaceSourceReaderSurface.vue", () => ({
  default: { name: "SpaceSourceReaderSurface", template: "<div data-reader-surface />" },
}));
vi.mock("../../../reading/presentation/use-reader-capture", () => ({
  useReaderCapture: () => ({
    canSave: { value: false },
    selectedAppendTarget: { value: undefined },
    existingNoteTargets: { value: [] },
    dropTargetAt: () => undefined,
    createNote: async () => {},
    createEvidence: async () => {},
    appendToNode: async () => {},
    askAboutPassage: async () => {},
    copyPassage: async () => {},
  }),
}));

describe("SpaceDocumentSource", () => {
  it("mounts a resizable splitter rail and keeps document source reading isolated from canvas state", async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    mocks.store.nodes = [createSpaceNode(createSpaceNotePayload("Source note"), { x: 0, y: 0 }, "source-1")];
    const target: SourceReadingTarget = { nodeId: "source-1", pane: "read", nonce: 1 };
    const wrapper = mount(SpaceDocumentSource, {
      props: { active: true, target },
      slots: { default: () => h("div", { "data-editor": "" }, "Editor") },
      global: {
        plugins: [
          pinia,
          i18n,
          [
            VueQueryPlugin,
            { queryClient: new QueryClient({ defaultOptions: { queries: { retry: false } } }) },
          ],
        ],
        provide: { [workspaceRightSidebarKey as symbol]: { open: () => {}, close: () => {} } },
      },
    });
    onTestFinished(() => wrapper.unmount());

    expect(wrapper.find("[data-editor]").exists()).toBe(true);
    expect(wrapper.find('[role="separator"]').exists()).toBe(true);
    expect(wrapper.find('[data-reader-host="document"]').exists()).toBe(true);
    expect(mocks.store.readingTarget).toBeUndefined();

    const closeButton = wrapper.find(`button[aria-label="${i18n.global.t("common.close")}"]`);
    expect(closeButton.exists()).toBe(true);
    await closeButton.trigger("click");

    expect(wrapper.emitted("close")).toHaveLength(1);
    expect(mocks.store.closeSourceReader).not.toHaveBeenCalled();
  });
});
