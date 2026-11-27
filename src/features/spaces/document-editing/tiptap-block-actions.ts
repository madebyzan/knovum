import { Extension, type Editor } from "@tiptap/core";
import type { Node as ProseMirrorNode } from "@tiptap/pm/model";
import { TextSelection } from "@tiptap/pm/state";

export type TurnIntoBlockKind =
  | "paragraph"
  | "heading1"
  | "heading2"
  | "heading3"
  | "bulletList"
  | "orderedList"
  | "taskList"
  | "blockquote"
  | "callout"
  | "codeBlock";

interface TopLevelBlockTarget {
  readonly index: number;
  readonly pos: number;
  readonly node: ProseMirrorNode;
}

function resolveTopLevelBlock(editor: Editor, explicitPos?: number): TopLevelBlockTarget | null {
  const { doc, selection } = editor.state;
  if (doc.childCount === 0) return null;
  const targetPos = explicitPos ?? selection.from;
  const clamped = Math.max(0, Math.min(targetPos, doc.content.size));
  const resolved = doc.resolve(clamped);
  const index = resolved.depth >= 1 ? resolved.index(0) : Math.min(resolved.index(0), doc.childCount - 1);
  if (index < 0 || index >= doc.childCount) return null;
  let pos = 0;
  for (let current = 0; current < index; current += 1) {
    pos += doc.child(current).nodeSize;
  }
  return { index, pos, node: doc.child(index) };
}

export function moveTopLevelBlockToIndex(editor: Editor, fromIndex: number, targetIndex: number): boolean {
  const { doc } = editor.state;
  if (fromIndex < 0 || fromIndex >= doc.childCount) return false;
  if (targetIndex < 0 || targetIndex > doc.childCount) return false;
  if (targetIndex === fromIndex || targetIndex === fromIndex + 1) return false;

  let fromPos = 0;
  let insertPos = 0;
  for (let current = 0; current < doc.childCount; current += 1) {
    const size = doc.child(current).nodeSize;
    if (current < fromIndex) fromPos += size;
    if (current < targetIndex) insertPos += size;
  }
  const node = doc.child(fromIndex);
  const tr = editor.state.tr.delete(fromPos, fromPos + node.nodeSize);
  const mappedInsert = tr.mapping.map(insertPos);
  tr.insert(mappedInsert, node);
  tr.setSelection(TextSelection.near(tr.doc.resolve(Math.min(mappedInsert + 1, tr.doc.content.size))));
  editor.view.dispatch(tr.scrollIntoView());
  return true;
}

export function moveTopLevelBlock(editor: Editor, direction: -1 | 1, explicitPos?: number): boolean {
  const target = resolveTopLevelBlock(editor, explicitPos);
  if (!target) return false;
  const targetIndex = direction === -1 ? target.index - 1 : target.index + 2;
  return moveTopLevelBlockToIndex(editor, target.index, targetIndex);
}

export function duplicateTopLevelBlock(editor: Editor, explicitPos?: number): boolean {
  const target = resolveTopLevelBlock(editor, explicitPos);
  if (!target) return false;
  const insertAt = target.pos + target.node.nodeSize;
  const copy = target.node.type.create(target.node.attrs, target.node.content, target.node.marks);
  const tr = editor.state.tr.insert(insertAt, copy);
  tr.setSelection(TextSelection.near(tr.doc.resolve(Math.min(insertAt + 1, tr.doc.content.size))));
  editor.view.dispatch(tr.scrollIntoView());
  return true;
}

export function deleteTopLevelBlock(editor: Editor, explicitPos?: number): boolean {
  const target = resolveTopLevelBlock(editor, explicitPos);
  if (!target) return false;
  const tr = editor.state.tr;
  const paragraph = editor.schema.nodes.paragraph;
  if (editor.state.doc.childCount === 1 && paragraph) {
    tr.replaceWith(target.pos, target.pos + target.node.nodeSize, paragraph.create());
  } else {
    tr.delete(target.pos, target.pos + target.node.nodeSize);
  }
  const focusPos = Math.min(target.pos + 1, tr.doc.content.size);
  tr.setSelection(TextSelection.near(tr.doc.resolve(Math.max(0, focusPos))));
  editor.view.dispatch(tr.scrollIntoView());
  return true;
}

export function selectTopLevelBlockText(
  editor: Editor,
  explicitPos?: number,
): { from: number; to: number } | null {
  const target = resolveTopLevelBlock(editor, explicitPos);
  if (!target) return null;
  const from = target.pos + 1;
  const to = target.pos + target.node.nodeSize - 1;
  if (to < from) return null;
  editor.chain().focus().setTextSelection({ from, to }).run();
  return { from, to };
}

export function insertParagraphBelowAndSlash(editor: Editor, explicitPos?: number): boolean {
  const target = resolveTopLevelBlock(editor, explicitPos);
  if (!target) return false;
  if (target.node.type.name === "paragraph" && target.node.content.size === 0) {
    return editor
      .chain()
      .focus()
      .setTextSelection(target.pos + 1)
      .insertContent("/")
      .run();
  }
  const insertAt = target.pos + target.node.nodeSize;
  return editor
    .chain()
    .focus()
    .insertContentAt(insertAt, { type: "paragraph" })
    .setTextSelection(insertAt + 1)
    .insertContent("/")
    .run();
}

export function turnTopLevelBlockInto(
  editor: Editor,
  kind: TurnIntoBlockKind,
  explicitPos?: number,
): boolean {
  const target = resolveTopLevelBlock(editor, explicitPos);
  if (!target) return false;
  const chain = editor
    .chain()
    .focus()
    .setTextSelection(Math.min(target.pos + 1, editor.state.doc.content.size));
  const runners: Record<TurnIntoBlockKind, () => boolean> = {
    paragraph: () =>
      (target.node.type.name === "callout" ? chain.lift("callout") : chain).setParagraph().run(),
    heading1: () => chain.setHeading({ level: 1 }).run(),
    heading2: () => chain.setHeading({ level: 2 }).run(),
    heading3: () => chain.setHeading({ level: 3 }).run(),
    bulletList: () => chain.toggleBulletList().run(),
    orderedList: () => chain.toggleOrderedList().run(),
    taskList: () => chain.toggleTaskList().run(),
    blockquote: () => chain.toggleBlockquote().run(),
    callout: () => chain.toggleCallout("note").run(),
    codeBlock: () => chain.toggleCodeBlock().run(),
  };
  return runners[kind]();
}

export const BlockKeyboardReorder = Extension.create({
  name: "blockKeyboardReorder",
  addKeyboardShortcuts() {
    return {
      "Alt-ArrowUp": () => moveTopLevelBlock(this.editor, -1),
      "Alt-ArrowDown": () => moveTopLevelBlock(this.editor, 1),
      "Mod-d": () => duplicateTopLevelBlock(this.editor),
    };
  },
});

export function createEditorErgonomicsExtension(onOpenAi?: () => void, onFocusTitle?: () => void) {
  return Extension.create({
    name: "spaceEditorErgonomics",
    addKeyboardShortcuts() {
      return {
        "Mod-j": () => (onOpenAi && this.editor.isEditable ? (onOpenAi(), true) : false),
        ArrowUp: () => {
          const { empty, from } = this.editor.state.selection;
          return onFocusTitle && empty && from <= 1 ? (onFocusTitle(), true) : false;
        },
        Backspace: () => {
          const { empty, from, $from } = this.editor.state.selection;
          if (!onFocusTitle || !empty || from !== 1) return false;
          if ($from.parent.type.name !== "paragraph" || $from.parent.textContent.length > 0) return false;
          return (onFocusTitle(), true);
        },
      };
    },
  });
}
