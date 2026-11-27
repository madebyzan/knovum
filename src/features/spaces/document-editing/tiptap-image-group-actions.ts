import type { Editor } from "@tiptap/core";
import type { Node as ProseMirrorNode } from "@tiptap/pm/model";
import { TextSelection } from "@tiptap/pm/state";

function resolveGroupNode(editor: Editor, getPos: () => number | undefined) {
  const pos = getPos();
  if (pos === undefined || editor.isDestroyed) return null;
  const node = editor.state.doc.nodeAt(pos);
  if (!node || node.type.name !== "imageGroup") return null;
  return { pos, node };
}

function collectChildImages(groupNode: ProseMirrorNode): ProseMirrorNode[] {
  const images: ProseMirrorNode[] = [];
  groupNode.forEach((child) => images.push(child));
  return images;
}

export function updateGroupImageAlt(
  editor: Editor,
  getPos: () => number | undefined,
  index: number,
  alt: string,
): boolean {
  const resolved = resolveGroupNode(editor, getPos);
  if (!resolved) return false;
  let offset = 1;
  for (let i = 0; i < resolved.node.childCount; i += 1) {
    const child = resolved.node.child(i);
    if (i === index) {
      const childPos = resolved.pos + offset;
      const tr = editor.state.tr.setNodeMarkup(childPos, undefined, { ...child.attrs, alt: alt || null });
      editor.view.dispatch(tr);
      return true;
    }
    offset += child.nodeSize;
  }
  return false;
}

export function reorderGroupImage(
  editor: Editor,
  getPos: () => number | undefined,
  fromIndex: number,
  toIndex: number,
): boolean {
  const resolved = resolveGroupNode(editor, getPos);
  if (!resolved || fromIndex === toIndex) return false;
  const images = collectChildImages(resolved.node);
  if (fromIndex < 0 || fromIndex >= images.length || toIndex < 0 || toIndex >= images.length) return false;
  const [moved] = images.splice(fromIndex, 1);
  if (!moved) return false;
  images.splice(toIndex, 0, moved);
  const nextGroup = resolved.node.type.create(resolved.node.attrs, images);
  editor.view.dispatch(
    editor.state.tr.replaceWith(resolved.pos, resolved.pos + resolved.node.nodeSize, nextGroup),
  );
  return true;
}

export function moveGroupImage(
  editor: Editor,
  getPos: () => number | undefined,
  index: number,
  delta: -1 | 1,
): boolean {
  return reorderGroupImage(editor, getPos, index, index + delta);
}

export function removeGroupImage(editor: Editor, getPos: () => number | undefined, index: number): boolean {
  const resolved = resolveGroupNode(editor, getPos);
  if (!resolved) return false;
  const images = collectChildImages(resolved.node);
  if (index < 0 || index >= images.length) return false;
  images.splice(index, 1);
  const from = resolved.pos;
  const to = resolved.pos + resolved.node.nodeSize;
  const tr = editor.state.tr;
  if (images.length === 0) tr.delete(from, to);
  else if (images.length === 1) tr.replaceWith(from, to, images[0]!);
  else tr.replaceWith(from, to, resolved.node.type.create(resolved.node.attrs, images));
  editor.view.dispatch(tr);
  return true;
}

export function splitImageGroupIntoParagraphs(editor: Editor, getPos: () => number | undefined): boolean {
  const resolved = resolveGroupNode(editor, getPos);
  if (!resolved) return false;
  const paragraphType = editor.state.schema.nodes.paragraph;
  if (!paragraphType) return false;
  const images = collectChildImages(resolved.node);
  const separated: ProseMirrorNode[] = [];
  images.forEach((img, idx) => {
    separated.push(img);
    if (idx < images.length - 1) separated.push(paragraphType.create());
  });
  const tr = editor.state.tr.replaceWith(resolved.pos, resolved.pos + resolved.node.nodeSize, separated);
  const cursorTarget = Math.min(resolved.pos + (images[0]?.nodeSize ?? 1) + 1, tr.doc.content.size);
  tr.setSelection(TextSelection.near(tr.doc.resolve(cursorTarget)));
  editor.view.dispatch(tr);
  editor.commands.focus();
  return true;
}
