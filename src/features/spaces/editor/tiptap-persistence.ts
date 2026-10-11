import { createKeyedSerialQueue } from "../../../foundation/async/keyed-serial-queue";
import { i18n } from "../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceMarkdownPayload, SpaceNotePayload, SpaceNodePayload } from "../model/nodes/node-payload";
import { tiptapContentSchema } from "../model/nodes/schema/content";
import { truncateUtf16 } from "../model/text";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import type { EditingState } from "./editing-state";

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

export interface PersistOptions {
  node: SpaceCanvasNode;
  save(change: NodePayloadChangeInput): Promise<SpaceNodePayload>;
  onSaved(payload: SpaceNodePayload): void;
  state: EditingState;
  editor?: PersistEditor | undefined;
}

interface SaveContext {
  state: EditingState;
  editor: PersistEditor;
}

export async function persistSpaceTiptapContent(options: PersistOptions): Promise<void> {
  const { node, state, editor } = options;
  if (!editor) throw new Error("Document editor is not ready to save");
  const pending = pendingSave(node, { state, editor });
  if (!pending) {
    options.onSaved(node.data);
    state.persistedRevision.value = state.revision.value;
    return;
  }
  return editorWrites.run(pending.id, async () => {
    const committed = await options.save({ id: pending.id, expectedData: node.data, data: pending.data });
    options.onSaved(committed);
    state.persistedRevision.value = Math.max(state.persistedRevision.value, pending.revision);
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
  const contentChanged = state.revision.value > state.persistedRevision.value;
  if (!contentChanged && node.data.title === title && node.data.text === text) return undefined;
  const tiptapContent = tiptapContentSchema.parse(editor.getJSON());
  return { id: node.id, data: { ...node.data, title, text, tiptapContent }, revision: state.revision.value };
}

function noteSave(node: NoteNode, context: SaveContext) {
  const { state, editor } = context;
  const contentChanged = state.revision.value > state.persistedRevision.value;
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
