import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { workspaceRightSidebarKey } from "../../../shared/contracts/workspace-right-sidebar";
import { applySpaceNodeContent, createSpaceNode } from "../canvas/node";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { loadSpaceSessionProjection } from "../session/space-projection";
import { useSpaceStore } from "../session/space-store";
import SpaceWorkbenchTabs from "../workspace/presentation/SpaceWorkbenchTabs.vue";
import { useWorkbenchTabCommands } from "../workspace/use-workbench-tab-commands";
import { createTestSpace } from "./support/session-context";

const repository = vi.hoisted(() => ({
  listSpaces: vi.fn<() => Promise<unknown[]>>(),
  markSpaceOpened: vi.fn<(spaceId: string) => Promise<void>>(),
  projection: vi.fn<typeof loadSpaceSessionProjection>(),
}));
vi.mock("../session/space-projection", () => ({ loadSpaceSessionProjection: repository.projection }));
vi.mock("../workspace/space-operations", () => ({
  createSpace: vi.fn<() => Promise<void>>(),
  listSpaces: repository.listSpaces,
  markSpaceOpened: repository.markSpaceOpened,
  updateSpaceTitle: vi.fn<() => Promise<void>>(),
}));
vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("../../../foundation/desktop/workspace-menu", () => ({
  syncWorkspaceMenuState: vi.fn<() => Promise<void>>(() => Promise.resolve()),
}));
vi.mock("vue-router", () => ({ useRoute: () => ({ name: "spaces" }) }));
vi.mock("../enrichment/use-enrichment-job-observation", () => ({ useEnrichmentJobObservation: () => {} }));
vi.mock("../canvas/thumbnail-scheduler", () => ({ captureActiveCanvasThumbnail: () => {} }));

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  repository.listSpaces.mockResolvedValue([createTestSpace()]);
  repository.markSpaceOpened.mockResolvedValue(undefined);
  repository.projection.mockResolvedValue({
    nodes: ["a", "b"].map((id) =>
      createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id),
    ) as SpaceCanvasNode[],
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
  });
});

async function setup() {
  const store = useSpaceStore();
  await store.initialize("Inbox");
  await store.openDocumentEditor("a");
  await store.openDocumentEditor("b");
  return store;
}

function mountStrip() {
  const wrapper = mount(SpaceWorkbenchTabs, {
    attachTo: document.body,
    global: {
      plugins: [i18n],
      provide: { [workspaceRightSidebarKey as symbol]: { open: () => {}, close: () => {} } },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("cycles the single tab list and closes the active tab with its shortcut", async () => {
  const store = await setup();
  let commands!: ReturnType<typeof useWorkbenchTabCommands>;
  const wrapper = mount(
    {
      setup() {
        commands = useWorkbenchTabCommands();
        return () => null;
      },
    },
    { global: { plugins: [i18n] } },
  );
  onTestFinished(() => wrapper.unmount());

  await commands.value.find(({ id }) => id === "spaces.next-tab")!.run();
  expect(store.surface.kind).toBe("canvas");
  await commands.value.find(({ id }) => id === "spaces.previous-tab")!.run();
  expect(store.surface).toEqual({ kind: "document", nodeId: "b" });

  await commands.value.find(({ id }) => id === "spaces.close-tab")!.run();
  expect(store.surface).toEqual({ kind: "document", nodeId: "a" });
  expect(store.tabs).toHaveLength(2);
});

it("keeps the Inbox tab unclosable and restores focus to the successor tab", async () => {
  const store = await setup();
  const wrapper = mountStrip();
  await flushPromises();

  expect(wrapper.findAll('[role="tab"]')).toHaveLength(3);
  expect(wrapper.get(`[data-tab-id="${store.tabs[0]!.id}"]`).find("button[aria-label]").exists()).toBe(false);

  const documents = store.editorTabs;
  await wrapper.get(`[data-tab-id="${documents[1]!.id}"] button[aria-label]`).trigger("click");
  await flushPromises();

  expect(store.surface).toEqual({ kind: "document", nodeId: "a" });
  expect(document.activeElement).toBe(wrapper.get('[role="tab"][data-state="active"]').element);
});

it("activates the neighbouring tab through tab-strip keyboard navigation", async () => {
  const store = await setup();
  const wrapper = mountStrip();
  await flushPromises();
  const active = wrapper.get('[role="tab"][data-state="active"]');
  (active.element as HTMLElement).focus();

  await active.trigger("keydown", { key: "ArrowLeft" });

  await vi.waitFor(() => expect(store.surface).toEqual({ kind: "document", nodeId: "a" }));
});

it("rechecks externally changed content after an in-flight save settles", async () => {
  const store = await setup();
  const tab = store.editorTabs[0]!;
  const session = useDocumentSessionStore().sessions.get(tab.id)!;
  session.saving.value = true;
  session.state.revision.value += 1;
  store.nodes = store.nodes.map((node) =>
    node.id === tab.nodeId ? applySpaceNodeContent(node, { ...node.data, title: "External" }) : node,
  );
  await flushPromises();
  expect(session.error.value).toBeUndefined();
  session.saving.value = false;
  await flushPromises();
  expect(session.error.value).toBe("conflict");
  expect(session.state.title.value).toBe("a");
});
