import { TextSelection } from "@tiptap/pm/state";
import type { Editor } from "@tiptap/vue-3";

import type { MathEditTarget } from "./tiptap-math";

export interface MathPopoverPosition {
  readonly left: number;
  readonly top: number;
}

export function resolveMathNodePos(editor: Editor, target: MathEditTarget): number | null {
  if (editor.isDestroyed) return null;
  const nodeName = target.kind === "block" ? "blockMath" : "inlineMath";
  const clamped = Math.max(0, Math.min(target.pos, editor.state.doc.content.size));
  const atPos = editor.state.doc.nodeAt(clamped);
  return atPos?.type.name === nodeName ? clamped : null;
}

export function measureMathPopoverPosition(
  editor: Editor,
  target: MathEditTarget,
): MathPopoverPosition | null {
  const pos = resolveMathNodePos(editor, target);
  if (pos === null) return null;
  const dom = editor.view.nodeDOM(pos);
  const rect =
    dom instanceof Element
      ? dom.getBoundingClientRect()
      : (() => {
          const coords = editor.view.coordsAtPos(pos);
          return { left: coords.left, right: coords.left + 80, top: coords.top, bottom: coords.bottom };
        })();
  const width = Math.min(440, globalThis.innerWidth - 24);
  const center = (rect.left + rect.right) / 2 - width / 2;
  const left = Math.min(Math.max(12, center), Math.max(12, globalThis.innerWidth - width - 12));
  const fitsBelow = rect.bottom + 110 <= globalThis.innerHeight - 12;
  const top = fitsBelow ? rect.bottom + 8 : Math.max(12, rect.top - 96);
  return { left: Math.round(left), top: Math.round(top) };
}

export function applyMathNodeMarkup(editor: Editor, pos: number, nextLatex: string, recordHistory: boolean) {
  const { state, view } = editor;
  const node = state.doc.nodeAt(pos);
  if (!node) return;
  const tr = state.tr.setNodeMarkup(pos, undefined, { ...node.attrs, latex: nextLatex });
  if (!recordHistory) tr.setMeta("addToHistory", false);
  view.dispatch(tr);
}

export function finalizeMathNodeEdit(
  editor: Editor,
  pos: number,
  initialLatex: string,
  trimmedLatex: string,
) {
  const node = editor.state.doc.nodeAt(pos);
  if (!trimmedLatex) {
    if (node) editor.view.dispatch(editor.state.tr.delete(pos, pos + node.nodeSize));
    return;
  }
  if (trimmedLatex !== initialLatex) {
    applyMathNodeMarkup(editor, pos, initialLatex, false);
    applyMathNodeMarkup(editor, pos, trimmedLatex, true);
  }
}

export function placeCursorAfterMathNode(editor: Editor, pos: number, kept: boolean, refocus: boolean) {
  if (editor.isDestroyed) return;
  const updatedNode = editor.state.doc.nodeAt(pos);
  const afterPos = Math.min(
    kept && updatedNode ? pos + updatedNode.nodeSize : pos,
    editor.state.doc.content.size,
  );
  const tr = editor.state.tr
    .setSelection(TextSelection.near(editor.state.doc.resolve(afterPos)))
    .setMeta("addToHistory", false);
  editor.view.dispatch(tr);
  if (refocus) editor.commands.focus();
}
