import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { createTestSpace } from "../../../__tests__/support/session-context";
import { createSpaceNode } from "../../../canvas/node";
import { createSpaceNotePayload } from "../../../document-editing/tiptap-content";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { useSpaceStore } from "../../../session/space-store";
import { createSpaceCanvasTab } from "../../../workspace/workbench-tab";
import SpaceSourceReader from "../SpaceSourceReader.vue";

const mocks = vi.hoisted(() => ({
  addedNodes: [] as Array<{ payload: SpaceNodePayload; position?: { x: number; y: number } }>,
  updatedContents: [] as Array<{ id: string; after: SpaceNodePayload }>,
  createdRelations: [] as Array<{ sourceId: string; targetId: string; kind: string }>,
  createdProvenance: [] as unknown[],
  selectedNodeIds: [] as string[],
  dockClosed: 0,
}));

vi.mock("../../../../../foundation/ipc/invoke-command", () => ({
  invokeCommand: async (command: string, args?: Record<string, unknown>) => {
    if (command === "plugin:spaces|create_provenance") return void mocks.createdProvenance.push(args?.input);
    return [];
  },
}));

vi.mock("../../../canvas/editor-session", async (importOriginal) => ({
  ...(await importOriginal()),
  currentCanvasEditor: () => ({
    // oxfmt-ignore
    getDocument: () => ({ nodes: [{ nodeId: "web-1", frame: { x: 100, y: 100, width: 300, height: 200, rotation: 0 }, sizingMode: "frame", zIndex: 0 }] }),
    canvasPointAt: () => ({ x: 420, y: 180 }),
  }),
  getCanvasEditorSession: () => ({
    selection: { value: { nodeIds: mocks.selectedNodeIds, structureNodeIds: [], relationIds: [] } },
    transitioning: { value: false },
    directManipulation: { value: undefined },
    activeTextEditorNodeId: { value: undefined },
    selectNodes: (ids: string[]) => void (mocks.selectedNodeIds = [...ids]),
    reveal: () => {},
  }),
}));

vi.mock("../../../canvas/use-context-dock", () => ({
  useContextDock: () => ({
    openAsk: () => {},
    openInspect: () => {},
    openSource: () => {},
    closeDock: () => void (mocks.dockClosed += 1),
  }),
}));
vi.mock("tauri-plugin-clipboard-api", () => ({ writeText: async () => {} }));

enableAutoUnmount(afterEach);

const makeNode = (id: string, data: SpaceNodePayload) => createSpaceNode(data, { x: 0, y: 0 }, id);

function findTextNode(root: Element, containing: string) {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let current = walker.nextNode(); current; current = walker.nextNode())
    if (current.textContent?.includes(containing)) return current;
  throw new Error(`No text node contains "${containing}"`);
}

async function selectPassage(root: Element, needle: string) {
  await flushPromises();
  const node = findTextNode(root, needle);
  const start = node.textContent!.indexOf(needle);
  const range = document.createRange();
  range.setStart(node, start);
  range.setEnd(node, start + needle.length);
  const rect = { left: 80, top: 120, right: 240, bottom: 140, width: 160, height: 20 } as DOMRect;
  range.getBoundingClientRect = () => rect;
  range.getClientRects = () => [rect] as unknown as DOMRectList;
  const selection = globalThis.getSelection()!;
  selection.removeAllRanges();
  selection.addRange(range);
  document.dispatchEvent(new Event("selectionchange"));
  await nextTick();
}

function mountReader() {
  const pinia = createPinia();
  setActivePinia(pinia);
  const store = useSpaceStore();
  // oxfmt-ignore
  const websiteNode = makeNode("web-1", { kind: "website", title: "Retrieval Paper", url: "https://example.com/retrieval", pageTitle: "Retrieval Paper", knowledgeRevision: "node1:web-rev", websiteSections: [{ heading: "Research Findings", text: "The proposed approach improves retrieval quality through hybrid indexing." }, { heading: "Experiment Results", text: "The experiment also indicates strong recall." }] });
  const tab = createSpaceCanvasTab("inbox");
  store.spaces = [createTestSpace({ id: "inbox", kind: "inbox", title: "Inbox" })];
  store.tabs = [tab];
  store.activeTabId = tab.id;
  store.rightSidebarView = "inspect";
  store.nodes = [websiteNode, makeNode("note-1", createSpaceNotePayload("Key Takeaways", "Initial note"))];
  store.recordHistoryMutation = async (fn: () => Promise<void>) => await fn();
  store.addContent = async (payload: SpaceNodePayload, position?: { x: number; y: number }) => {
    mocks.addedNodes.push({ payload, ...(position ? { position } : {}) });
    const created = makeNode(`created-${mocks.addedNodes.length}`, payload);
    store.nodes = [...store.nodes, created];
    return [created];
  };
  store.updateNodeContents = async (patches: readonly { id: string; after: SpaceNodePayload }[]) => {
    mocks.updatedContents.push(...patches);
  };
  store.createRelation = (async (sourceId: string, targetId: string, kind: string) => {
    mocks.createdRelations.push({ sourceId, targetId, kind });
  }) as unknown as typeof store.createRelation;
  store.openSourceReader({ nodeId: "web-1" });

  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(SpaceSourceReader, {
    attachTo: document.body,
    global: { plugins: [pinia, i18n, [VueQueryPlugin, { queryClient }]] },
  });
  const bodyEl = wrapper.get("[data-source-reader] > div.relative").element as HTMLElement;
  bodyEl.getBoundingClientRect = () =>
    ({ left: 0, top: 40, right: 500, bottom: 700, width: 500, height: 660 }) as DOMRect;
  return { wrapper, store };
}

describe("SpaceSourceReader Read & Organize flow", () => {
  beforeEach(() => {
    mocks.addedNodes = [];
    mocks.updatedContents = [];
    mocks.createdRelations = [];
    mocks.createdProvenance = [];
    mocks.selectedNodeIds = [];
    mocks.dockClosed = 0;
  });

  afterEach(() => {
    globalThis.getSelection()?.removeAllRanges();
    vi.clearAllMocks();
  });

  it("auto-closes the redundant inspect sidebar and creates Note/Evidence with section locators", async () => {
    const { wrapper } = mountReader();
    expect(mocks.dockClosed).toBe(1);

    await selectPassage(
      wrapper.element,
      "The proposed approach improves retrieval quality through hybrid indexing.",
    );
    const saveLabel = i18n.global.t("spaces.conversation.capture.saveSelection");
    await wrapper.get(`[role="toolbar"] [aria-label="${saveLabel}"]`).trigger("click");
    await flushPromises();

    expect(mocks.addedNodes[0]?.payload.kind).toBe("note");
    expect(mocks.createdRelations[0]).toEqual({
      sourceId: "created-1",
      targetId: "web-1",
      kind: "derives-from",
    });
    expect(mocks.createdProvenance[0]).toMatchObject({
      targetId: "created-1",
      sourceNodeId: "web-1",
      sourceRevision: "node1:web-rev",
      locator: { location: { kind: "website", section: "Research Findings", startOffset: 0, endOffset: 73 } },
    });

    await selectPassage(wrapper.element, "The experiment also indicates strong recall.");
    await wrapper
      .get(`[aria-label="${i18n.global.t("spaces.conversation.evidence.extract")}"]`)
      .trigger("click");
    await flushPromises();

    expect(mocks.addedNodes[1]?.payload).toMatchObject({
      kind: "object",
      properties: { observation: "The experiment also indicates strong recall." },
    });
  });

  it("opens an explicit note picker before appending when no canvas note is selected", async () => {
    const { wrapper } = mountReader();
    await selectPassage(wrapper.element, "The experiment also indicates strong recall.");
    const appendLabel = i18n.global.t("spaces.conversation.capture.appendTo", { title: "…" });
    await wrapper.get(`[aria-label="${appendLabel}"]`).trigger("click");
    await nextTick();

    // First click reveals the picker rather than silently appending to an unselected note.
    expect(mocks.updatedContents).toHaveLength(0);
    const pickerButton = wrapper
      .findAll('[role="toolbar"] button')
      .find((button) => button.text().includes("Key Takeaways"));
    expect(pickerButton).toBeDefined();
    await pickerButton!.trigger("click");
    await flushPromises();

    expect(mocks.updatedContents[0]?.id).toBe("note-1");
    expect(mocks.createdProvenance[0]).toMatchObject({ targetId: "note-1", sourceNodeId: "web-1" });
  });
});
