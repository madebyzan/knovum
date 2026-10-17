import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { workspaceRightSidebarKey } from "../../../../../shared/contracts/workspace-right-sidebar";
import { createSpaceNode } from "../../../canvas/node";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import type { SpaceEditorTab } from "../../../workspace/workbench-tab";
import { useDocumentSessionStore } from "../../document-session-store";
import { createSpaceNotePayload } from "../../tiptap-content";
import SpaceTiptapEditor from "../SpaceTiptapEditor.vue";

type ContentChange = { id: string; data: SpaceNodePayload };

const mocks = vi.hoisted(() => ({
  store: {
    saveNodeContentForSpace:
      vi.fn<(spaceId: string, change: { id: string; data: unknown }) => Promise<unknown>>(),
    discardUnavailableEditorTab: vi.fn<(tabId: string) => Promise<void>>(),
    editorLocateRequest: undefined as unknown,
  },
}));
vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => mocks.store }));
vi.mock("../../../canvas/use-context-dock", () => ({ useContextDock: () => ({}) }));
vi.mock("../../tiptap-text-statistics", () => ({
  createTiptapTextStatistics: () => ({
    wordCount: 0,
    characterCount: 0,
    update: () => {},
    dispose: () => {},
  }),
}));
vi.mock("../../../../../foundation/file-drop/events", () => ({
  isFileDropInside: () => false,
  listenForFileDrops: vi.fn<() => Promise<() => void>>(() => Promise.resolve(() => {})),
}));
vi.mock("../../tiptap-extensions", async () => {
  const { StarterKit } = await import("@tiptap/starter-kit");
  const { Markdown } = await import("@tiptap/markdown");
  return { createSpaceTiptapExtensions: () => [StarterKit, Markdown] };
});

const tab: SpaceEditorTab = { id: "document-tab", kind: "document", nodeId: "node", spaceId: "space" };

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  mocks.store.editorLocateRequest = shallowRef();
  mocks.store.saveNodeContentForSpace.mockImplementation(async (_spaceId, change) =>
    Promise.resolve((change as ContentChange).data),
  );
  useDocumentSessionStore().open(
    tab,
    createSpaceNode(createSpaceNotePayload("Draft"), { x: 0, y: 0 }, tab.nodeId),
  );
});

function render() {
  const errors: unknown[] = [];
  const wrapper = mount(SpaceTiptapEditor, {
    attachTo: document.body,
    props: { tab, active: true },
    global: {
      plugins: [i18n],
      provide: { [workspaceRightSidebarKey as symbol]: { open: () => {}, close: () => {} } },
      config: { errorHandler: (error: unknown) => errors.push(error) },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return { wrapper, errors };
}

/**
 * The bubble menu hands its root element to the ProseMirror plugin, which re-parents it outside this
 * component's tree. Mounting it on surface visibility desynchronized the virtual and the real DOM,
 * so every later patch threw inside the renderer.
 */
it("keeps the editor subtree renderable while its surface is hidden and shown again", async () => {
  const { wrapper, errors } = render();
  await vi.waitFor(() => expect(wrapper.find(".tiptap").exists()).toBe(true));
  const content = wrapper.get(".tiptap").element;

  await wrapper.setProps({ active: false });
  await flushPromises();
  await wrapper.setProps({ active: true });
  await flushPromises();

  expect(errors).toEqual([]);
  expect(wrapper.get(".tiptap").element).toBe(content);
  expect(() => wrapper.unmount()).not.toThrow();
});

it("offers no close or back affordance inside the editor header", async () => {
  const { wrapper } = render();
  await vi.waitFor(() => expect(wrapper.find(".tiptap").exists()).toBe(true));
  const labels = wrapper
    .get("header")
    .findAll("button")
    .map((button) => button.attributes("aria-label"));

  expect(labels).not.toContain(i18n.global.t("common.back"));
  expect(labels).not.toContain(i18n.global.t("common.close"));
});
