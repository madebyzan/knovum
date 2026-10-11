import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { TabsRoot } from "reka-ui";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { h } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type * as CanvasSession from "../canvas/editor-session";
import { applySpaceNodeContent, createSpaceNode } from "../canvas/node";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { useSpaceStore } from "../session/space-store";
import SpaceWorkTabs from "../workspace/presentation/SpaceWorkTabs.vue";
import { useWorkspaceTabCommands } from "../workspace/use-workspace-tab-commands";

const mocks = vi.hoisted(() => ({ focus: vi.fn<() => void>() }));
vi.mock("../canvas/editor-session", async (original) => ({
  ...(await original<typeof CanvasSession>()),
  currentCanvasEditor: () => ({ focus: mocks.focus }),
}));

vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("../enrichment/use-enrichment-job-observation", () => ({ useEnrichmentJobObservation: () => {} }));
beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
});

async function setup() {
  const store = useSpaceStore();
  store.activeSpaceId = "inbox";
  store.openSpaceIds = ["inbox"];
  store.nodes = ["a", "b"].map((id) => createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id));
  await store.openDocumentEditor("a");
  await store.openDocumentEditor("b");
  return store;
}

it("cycles work items and closes the active editor while retaining the only Space", async () => {
  const store = await setup();
  let commands!: ReturnType<typeof useWorkspaceTabCommands>;
  const wrapper = mount(
    {
      setup() {
        commands = useWorkspaceTabCommands();
        return () => null;
      },
    },
    { global: { plugins: [i18n] } },
  );
  onTestFinished(() => wrapper.unmount());
  await commands.value.find(({ id }) => id === "spaces.next-work-tab")!.run();
  expect(store.surface.kind).toBe("canvas");
  await commands.value.find(({ id }) => id === "spaces.previous-work-tab")!.run();
  expect(store.surface).toEqual({ kind: "document", nodeId: "b" });
  await commands.value.find(({ id }) => id === "spaces.close-tab")!.run();
  expect(store.surface).toEqual({ kind: "document", nodeId: "a" });
  expect(store.editorTabs).toHaveLength(1);
  expect(store.openSpaceIds).toHaveLength(1);
});

it("uses tab keyboard navigation and restores focus after closing the last editor", async () => {
  const store = await setup();
  await store.returnToCanvas();
  const wrapper = mount(
    {
      setup() {
        return () =>
          h(
            TabsRoot,
            {
              modelValue: store.activeWorkItems.get("inbox") ?? "canvas",
              "onUpdate:modelValue": (id) =>
                id === "canvas" ? store.returnToCanvas() : store.activateEditorTab(String(id)),
            },
            () => h(SpaceWorkTabs),
          );
      },
    },
    { attachTo: document.body, global: { plugins: [i18n] } },
  );
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  const canvas = wrapper.get('[data-work-tab="canvas"]');
  (canvas.element as HTMLElement).focus();
  await canvas.trigger("keydown", { key: "ArrowRight" });
  await vi.waitFor(() => expect(store.surface).toEqual({ kind: "document", nodeId: "a" }));
  const firstTab = store.editorTabs[0]!;
  await wrapper.get(`[data-work-tab="${firstTab.id}"] button[aria-label]`).trigger("click");
  await flushPromises();
  expect(store.surface).toEqual({ kind: "document", nodeId: "b" });
  expect(document.activeElement).toBe(wrapper.get('[role="tab"][data-state="active"]').element);
  const lastTab = store.editorTabs[0]!;
  await wrapper.get(`[data-work-tab="${lastTab.id}"] button[aria-label]`).trigger("click");
  await flushPromises();
  expect(store.surface.kind).toBe("canvas");
  expect(wrapper.find('[role="tablist"]').exists()).toBe(false);
  expect(mocks.focus).toHaveBeenCalledOnce();
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
