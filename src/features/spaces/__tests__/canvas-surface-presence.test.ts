import { shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { expect, it, onTestFinished, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { workspaceRightSidebarKey } from "../../../shared/contracts/workspace-right-sidebar";
import { useCanvasToolStore } from "../canvas/canvas-tool-store";
import type { CanvasEditorPort } from "../canvas/contracts/canvas-editor-port";
import { registerCanvasEditor } from "../canvas/editor-session";
import CanvasSurfaceHost from "../canvas/host/CanvasSurfaceHost.vue";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import SpaceCanvas from "../presentation/canvas/SpaceCanvas.vue";
import SpaceCanvasProductLayer from "../presentation/canvas/SpaceCanvasProductLayer.vue";
import { useSpaceStore } from "../session/space-store";

vi.mock("../enrichment/use-enrichment-job-observation", () => ({ useEnrichmentJobObservation: () => {} }));
vi.mock("../presentation/canvas/use-space-canvas-imports", () => ({
  useSpaceCanvasImports: () => ({
    fileHover: { value: false },
    error: { value: undefined },
    fileHoverPosition: { value: undefined },
  }),
}));

it("treats a Section-only canvas as populated and suspends hidden canvas input", async () => {
  const pinia = createPinia();
  setActivePinia(pinia);
  const store = useSpaceStore();
  store.activeSpaceId = "inbox";
  onTestFinished(
    registerCanvasEditor({
      getViewport: () => ({ x: 0, y: 0, zoom: 1 }),
      getSelectionSnapshot: () => ({
        nodeIds: [],
        structureNodeIds: [],
        relationIds: [],
        nodePlacements: [],
        members: [],
      }),
    } as unknown as CanvasEditorPort),
  );
  const wrapper = shallowMount(SpaceCanvas, {
    props: { renderingEnabled: true },
    global: { plugins: [pinia, i18n], provide: { [workspaceRightSidebarKey as symbol]: { open: () => {} } } },
  });
  onTestFinished(() => wrapper.unmount());
  const host = wrapper.findComponent(CanvasSurfaceHost);
  const chrome = wrapper.findComponent(SpaceCanvasProductLayer);
  expect(host.props("navigationEnabled")).toBe(false);
  expect(chrome.props("empty")).toBe(true);
  store.urlCaptures = [
    {
      id: "capture",
      spaceId: "elsewhere",
      url: "https://example.com",
      title: "Capture",
      position: { x: 0, y: 0 },
      status: "preparing",
      downloadedBytes: 0,
      totalBytes: null,
      processedMs: null,
      durationMs: null,
    },
  ];
  await wrapper.vm.$nextTick();
  expect(chrome.props("empty")).toBe(true);
  expect(chrome.props("captures")).toEqual([]);
  await useCanvasToolStore().selectTool("section");
  await wrapper.vm.$nextTick();
  expect(chrome.props("empty")).toBe(false);
  useCanvasToolStore().resetTool();
  store.editSnapshot = {
    editRevision: 1,
    layouts: [],
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Research" },
    ],
  };
  await wrapper.vm.$nextTick();
  expect(host.props("navigationEnabled")).toBe(true);
  expect(store.nodes).toEqual([]);
  expect(chrome.props("empty")).toBe(false);
  store.nodes = [
    { ...createSpaceNode(createSpaceNotePayload("Hidden"), { x: 10000, y: 10000 }, "hidden"), hidden: true },
  ];
  store.editSnapshot = {
    ...store.editSnapshot,
    structure: [
      { id: "content", kind: "content", nodeId: "hidden", parentId: null, order: 0, locked: false },
    ],
  };
  await wrapper.vm.$nextTick();
  expect(chrome.props("empty")).toBe(false);
  expect(host.props("navigationEnabled")).toBe(true);
  await wrapper.setProps({ renderingEnabled: false });
  expect(host.props("interactionEnabled")).toBe(false);
  expect(wrapper.attributes("inert")).toBeDefined();
  expect((wrapper.element as HTMLElement).style.display).toBe("none");
  useCanvasToolStore().resetTool();
});
