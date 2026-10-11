import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { nextTick, shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { createSpaceNode } from "../canvas/node";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import { useSpaceTiptapEditor } from "../document-editing/use-space-tiptap-editor";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import { prepareEditorSessionClose } from "../workspace/session-participants";
import { deferred } from "./support/canvas-fixtures";

const mocks = vi.hoisted(() => ({
  store: {
    saveNodeContentForSpace:
      vi.fn<(spaceId: string, change: NodePayloadChangeInput) => Promise<SpaceNodePayload>>(),
    returnToCanvas: vi.fn<() => Promise<void>>(),
  },
}));
vi.mock("../session/space-store", () => ({ useSpaceStore: () => mocks.store }));
vi.mock("../canvas/use-context-dock", () => ({ useContextDock: () => ({}) }));
vi.mock("../document-editing/tiptap-text-statistics", () => ({
  createTiptapTextStatistics: () => ({
    wordCount: 0,
    characterCount: 0,
    update: () => {},
    dispose: () => {},
  }),
}));
vi.mock("../document-editing/tiptap-extensions", async () => {
  const { StarterKit } = await import("@tiptap/starter-kit");
  const { Markdown } = await import("@tiptap/markdown");
  return { createSpaceTiptapExtensions: () => [StarterKit, Markdown] };
});

let wrapper: ReturnType<typeof mount> | undefined;
beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  mocks.store.saveNodeContentForSpace.mockImplementation(async (_spaceId, change) => change.data);
});
afterEach(() => wrapper?.unmount());

async function openDocument(kind: "note" | "markdown" = "note") {
  const tab = { id: "document-tab", kind: "document", nodeId: "node", spaceId: "origin" } as const;
  const node = createSpaceNode(
    kind === "note" ? createSpaceNotePayload("Original") : { kind: "markdown", title: "Original", text: "" },
    { x: 0, y: 0 },
    tab.nodeId,
  );
  const sessions = useDocumentSessionStore();
  sessions.open(tab, node);
  const active = shallowRef(true);
  let document!: ReturnType<typeof useSpaceTiptapEditor>;
  wrapper = mount(
    {
      setup() {
        document = useSpaceTiptapEditor(tab, active);
        return () => null;
      },
    },
    { global: { plugins: [createI18n({ legacy: false, locale: "en", messages: { en: {} } })] } },
  );
  await flushPromises();
  return { tab, node, active, document, session: sessions.sessions.get(tab.id)! };
}

it("retains the editor, title, cursor and undo history while its tab is inactive", async () => {
  const { active, document } = await openDocument();
  const editor = document.editor.value!;
  editor.commands.insertContent("Draft");
  document.title.value = "Draft title";
  editor.commands.setTextSelection(3);
  active.value = false;
  await nextTick();
  expect(editor.isEditable).toBe(false);
  expect(document.editor.value).toBe(editor);
  expect(editor.state.selection.from).toBe(3);
  expect(document.title.value).toBe("Draft title");
  expect(mocks.store.saveNodeContentForSpace).not.toHaveBeenCalled();
  active.value = true;
  await nextTick();
  expect(editor.isEditable).toBe(true);
  expect(editor.commands.undo()).toBe(true);
  expect(editor.getText()).toBe("");
});

it("saves to its captured Space and keeps edits made during that write dirty", async () => {
  const { document, session } = await openDocument();
  const completion = deferred<SpaceNodePayload>();
  mocks.store.saveNodeContentForSpace.mockReturnValueOnce(completion.promise);
  document.editor.value!.commands.insertContent("First");
  const saving = document.persist();
  await vi.waitFor(() => expect(mocks.store.saveNodeContentForSpace).toHaveBeenCalledOnce());
  const [spaceId, change] = mocks.store.saveNodeContentForSpace.mock.calls[0]!;
  expect(spaceId).toBe("origin");
  expect(change.id).toBe("node");
  document.editor.value!.commands.insertContent(" second");
  completion.resolve(change.data);
  await saving;
  expect(session.node.value.data).toEqual(change.data);
  expect(session.dirty.value).toBe(true);
  await document.persist();
  expect(session.dirty.value).toBe(false);
});

it("retains dirty content when persistence fails", async () => {
  const { document, session } = await openDocument();
  document.editor.value!.commands.insertContent("Unsaved");
  const failure = new Error("disk full");
  mocks.store.saveNodeContentForSpace.mockRejectedValueOnce(failure);
  await expect(document.persist()).rejects.toBe(failure);
  expect(document.editor.value!.getText()).toBe("Unsaved");
  expect(session.dirty.value).toBe(true);
  expect(document.saveStatus.value).toBe("unsaved");
});

it("saves Markdown text and its rich document together for reopening", async () => {
  const { document, session } = await openDocument("markdown");
  document.editor.value!.commands.insertContent("Draft");
  await document.persist();
  expect(session.node.value.data).toMatchObject({
    kind: "markdown",
    text: "Draft",
    tiptapContent: document.editor.value!.getJSON(),
  });
  expect(session.dirty.value).toBe(false);
});

it("pauses autosave while hidden and resumes saving to the owning Space", async () => {
  const { document, active } = await openDocument();
  vi.useFakeTimers();
  onTestFinished(() => {
    vi.useRealTimers();
  });
  document.editor.value!.commands.insertContent("Draft");
  active.value = false;
  await nextTick();
  await vi.advanceTimersByTimeAsync(1500);
  expect(mocks.store.saveNodeContentForSpace).not.toHaveBeenCalled();
  active.value = true;
  await nextTick();
  await vi.advanceTimersByTimeAsync(1000);
  expect(mocks.store.saveNodeContentForSpace).toHaveBeenCalledOnce();
  expect(mocks.store.saveNodeContentForSpace.mock.calls[0]![0]).toBe("origin");
});

it("drains edits made during an admitted save before accepting a close", async () => {
  const { document, session } = await openDocument();
  const completion = deferred<SpaceNodePayload>();
  mocks.store.saveNodeContentForSpace.mockReturnValueOnce(completion.promise);
  document.editor.value!.commands.insertContent("First");
  const closing = prepareEditorSessionClose({ tabIds: [session.tab.id] });
  await vi.waitFor(() => expect(mocks.store.saveNodeContentForSpace).toHaveBeenCalledOnce());
  document.editor.value!.commands.insertContent(" second");
  completion.resolve(mocks.store.saveNodeContentForSpace.mock.calls[0]![1].data);
  await closing;
  expect(mocks.store.saveNodeContentForSpace).toHaveBeenCalledTimes(2);
  expect(session.dirty.value).toBe(false);
  expect(document.editor.value!.getText()).toBe("First second");
});

it("settles saving state and retains the draft when document serialization fails", async () => {
  const { document, session } = await openDocument();
  document.editor.value!.commands.insertContent("Draft");
  vi.spyOn(document.editor.value!, "getJSON").mockImplementation(() => {
    throw new Error("Document serialization failed");
  });
  await expect(document.persist()).rejects.toThrow("Document serialization failed");
  expect(session.saving.value).toBe(false);
  expect(session.dirty.value).toBe(true);
  expect(mocks.store.saveNodeContentForSpace).not.toHaveBeenCalled();
});
