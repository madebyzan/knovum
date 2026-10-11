import { ref, type Ref } from "vue";

import { createKeyedSerialQueue } from "../../../foundation/async/keyed-serial-queue";
import { i18n } from "../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMarkdownPayload, SpaceNotePayload } from "../model/nodes/node-payload";
import { tiptapContentSchema } from "../model/nodes/schema/content";
import { truncateUtf16 } from "../model/text";
import type { useSpaceStore } from "../session/space-store";
import { emptyTiptapContent } from "./tiptap-content";
import type { TiptapJsonNode } from "./tiptap-document";

type MarkdownNode = { id: string; data: SpaceMarkdownPayload };
type NoteNode = { id: string; data: SpaceNotePayload };
type PersistPayload = SpaceMarkdownPayload | SpaceNotePayload;
type PersistEditor = {
  getMarkdown(): string;
  getJSON(): unknown;
  state: {
    doc: { content: { size: number }; textBetween: (f: number, t: number, b: string, l: string) => string };
  };
};

const editorWrites = createKeyedSerialQueue<string>();

export interface EditingState {
  title: Ref<string>;
  markdown: Ref<string>;
  tiptapContent: Ref<TiptapJsonNode>;
  plainText: Ref<string>;
  revision: Ref<number>;
  persistedRevision: number;
  selection: Ref<number>;
  loadedId: string | undefined;
}

export function createEditingState(): EditingState {
  return {
    title: ref(""),
    markdown: ref(""),
    tiptapContent: ref(emptyTiptapContent()),
    plainText: ref(""),
    revision: ref(0),
    persistedRevision: 0,
    selection: ref(0),
    loadedId: undefined,
  };
}

export interface PersistOptions {
  store: Pick<ReturnType<typeof useSpaceStore>, "saveNodeContent">;
  nodes: readonly SpaceCanvasNode[];
  state: EditingState;
  editor?: PersistEditor | undefined;
}

interface SaveContext {
  state: EditingState;
  editor: PersistEditor;
}

export function persistSpaceTiptapContent(options: PersistOptions): Promise<void> {
  const { store, nodes, state, editor } = options;
  const node = state.loadedId ? nodes.find((candidate) => candidate.id === state.loadedId) : undefined;
  if (!node) return Promise.resolve();
  const pending = editor ? pendingSave(node, { state, editor }) : undefined;
  if (!pending) return Promise.resolve();
  return editorWrites.run(pending.id, async () => {
    await store.saveNodeContent({ id: pending.id, expectedData: node.data, data: pending.data });
    state.persistedRevision = Math.max(state.persistedRevision, pending.revision);
  });
}

function pendingSave(
  node: SpaceCanvasNode,
  context: SaveContext,
): { id: string; data: PersistPayload; revision: number } | undefined {
  if (node.data.kind === "markdown") return markdownSave({ id: node.id, data: node.data }, context);
  if (node.data.kind === "note") return noteSave({ id: node.id, data: node.data }, context);
  return undefined;
}

function markdownSave(node: MarkdownNode, context: SaveContext) {
  const { state, editor } = context;
  const text = editor.getMarkdown();
  const title =
    state.title.value.trim() || firstLine(text) || i18n.global.t("spaces.editor.markdown.untitled");
  if (node.data.title === title && node.data.text === text) return undefined;
  return { id: node.id, data: { ...node.data, title, text }, revision: state.revision.value };
}

function noteSave(node: NoteNode, context: SaveContext) {
  const { state, editor } = context;
  const contentChanged = state.revision.value > state.persistedRevision;
  const plainText = editor.state.doc.textBetween(0, editor.state.doc.content.size, "\n", "\n");
  const title = state.title.value.trim() || firstLine(plainText) || i18n.global.t("spaces.editor.note.new");
  if (!contentChanged && node.data.title === title && node.data.plainText === plainText) return undefined;
  const tiptapContent = tiptapContentSchema.parse(editor.getJSON());
  return {
    id: node.id,
    data: { ...node.data, title, tiptapContent, plainText },
    revision: state.revision.value,
  };
}

function firstLine(value: string) {
  const line =
    value
      .trim()
      .replace(/^#{1,6}\s+/u, "")
      .split(/\r?\n/u, 1)[0] ?? "";
  return truncateUtf16(line, 80);
}
