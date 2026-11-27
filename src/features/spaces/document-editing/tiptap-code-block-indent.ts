import { TextSelection } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";

const INDENT = "  ";

function removeLeadingIndent(line: string): { readonly text: string; readonly removed: number } {
  if (line.startsWith(INDENT)) return { text: line.slice(INDENT.length), removed: INDENT.length };
  if (line.startsWith("\t") || line.startsWith(" ")) return { text: line.slice(1), removed: 1 };
  return { text: line, removed: 0 };
}

function transformCodeLines(lines: readonly string[], outdent: boolean) {
  let firstDelta = 0;
  let totalDelta = 0;
  const updated = lines.map((line, index) => {
    if (!outdent) {
      if (index === 0) firstDelta = INDENT.length;
      totalDelta += INDENT.length;
      return `${INDENT}${line}`;
    }
    const trimmed = removeLeadingIndent(line);
    if (index === 0) firstDelta = -trimmed.removed;
    totalDelta -= trimmed.removed;
    return trimmed.text;
  });
  return { replacement: updated.join("\n"), firstDelta, totalDelta };
}

function resolveSelectedLineSpan(blockText: string, startOffset: number, endOffset: number, empty: boolean) {
  const lineStart = blockText.lastIndexOf("\n", Math.max(0, startOffset - 1)) + 1;
  const trimTrailing = !empty && endOffset > startOffset && blockText[endOffset - 1] === "\n";
  const nextNewline = blockText.indexOf("\n", trimTrailing ? endOffset - 1 : endOffset);
  const lineEnd = nextNewline === -1 ? blockText.length : nextNewline;
  return { lineStart, lineEnd };
}

export function handleCodeBlockIndent(view: EditorView, outdent: boolean): boolean {
  const { state } = view;
  const { $from, $to, from, to, empty } = state.selection;
  if ($from.parent.type.name !== "codeBlock" || !$from.sameParent($to)) return false;

  if (empty && !outdent) {
    view.dispatch(state.tr.insertText(INDENT, from, to).scrollIntoView());
    return true;
  }

  const blockStart = $from.start();
  const blockText = $from.parent.textContent;
  const { lineStart, lineEnd } = resolveSelectedLineSpan(
    blockText,
    from - blockStart,
    to - blockStart,
    empty,
  );
  const segment = blockText.slice(lineStart, lineEnd);
  const { replacement, firstDelta, totalDelta } = transformCodeLines(segment.split("\n"), outdent);
  if (replacement === segment) return true;

  const tr = state.tr.insertText(replacement, blockStart + lineStart, blockStart + lineEnd);
  const nextFrom = Math.max(blockStart + lineStart, from + firstDelta);
  const nextTo = empty ? nextFrom : Math.max(nextFrom, to + totalDelta);
  tr.setSelection(TextSelection.create(tr.doc, nextFrom, nextTo));
  view.dispatch(tr.scrollIntoView());
  return true;
}
