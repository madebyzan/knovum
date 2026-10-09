import type { Editor } from "@tiptap/vue-3";

import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import { setReasoningTraceRange } from "./tiptap-reasoning-trace";

type TransformSource = { text: string; from: number; to: number } | { text: string; insertAt: number };

export function markReasoning(editor: Editor, source: TransformSource) {
  if (!("from" in source)) return;
  setReasoningTraceRange(editor.view, source.from, source.to);
}

export function actionSource(editor: Editor, action: EditorTransformAction): TransformSource | undefined {
  return action === "complete" ? completionSource(editor) : selectionSource(editor);
}

function selectionSource(editor: Editor): TransformSource | undefined {
  const { from, to } = editor.state.selection;
  return from === to ? undefined : { from, to, text: editor.state.doc.textBetween(from, to, "\n", "\n") };
}

function completionSource(editor: Editor): TransformSource | undefined {
  const { from, to, $from } = editor.state.selection;
  if (from !== to) return { insertAt: to, text: editor.state.doc.textBetween(from, to, "\n", "\n") };
  const blockStart = $from.start($from.depth);
  const text = editor.state.doc.textBetween(blockStart, from, "\n", "\n");
  return text.trim() ? { insertAt: from, text } : undefined;
}

export function applyGeneratedText(
  editor: Editor,
  action: EditorTransformAction,
  source: TransformSource,
  generated: string,
) {
  if (action === "complete" && "insertAt" in source) {
    return editor
      .chain()
      .focus()
      .setTextSelection(source.insertAt)
      .insertContent(generated, { contentType: "markdown" })
      .run();
  }
  if (!("from" in source)) return false;
  return editor
    .chain()
    .focus()
    .insertContentAt({ from: source.from, to: source.to }, generated, { contentType: "markdown" })
    .run();
}
