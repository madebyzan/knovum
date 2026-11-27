import { InputRule, type Editor } from "@tiptap/core";
import type { NodeType } from "@tiptap/pm/model";
import { NodeSelection } from "@tiptap/pm/state";
import { renderToString } from "katex";

export function validateKatexSyntax(latex: string, displayMode: boolean): string | null {
  const source = latex.trim();
  if (!source) return null;
  try {
    renderToString(source, { throwOnError: true, strict: false, displayMode });
    return null;
  } catch (caught) {
    return caught instanceof Error ? caught.message.replace(/^KaTeX parse error:\s*/u, "") : String(caught);
  }
}

export interface MathEditTarget {
  readonly kind: "inline" | "block";
  readonly pos: number;
  readonly latex: string;
  readonly createdEmpty?: boolean;
}

function isLikelyInlineLatex(candidate: string): boolean {
  if (!candidate || /^\s|\s$/u.test(candidate)) return false;
  if (/^\d+(?:[.,]\d+)?$/u.test(candidate)) return false;
  return true;
}

function findSingleDollarMath(text: string) {
  const match = /\$([^$\n]+?)\$$/u.exec(text);
  if (!match) return null;
  const start = match.index;
  if (start > 0 && (text[start - 1] === "$" || text[start - 1] === "\\")) return null;
  const latex = match[1] ?? "";
  if (!isLikelyInlineLatex(latex)) return null;
  return { index: start, text: match[0], data: { latex } };
}

function findDoubleDollarInlineMath(text: string) {
  const match = /\$\$([^$\n]+?)\$\$$/u.exec(text);
  if (!match) return null;
  const start = match.index;
  if (start === 0 || text[start - 1] === "$") return null;
  const latex = (match[1] ?? "").trim();
  if (!latex) return null;
  return { index: start, text: match[0], data: { latex } };
}

export function createInlineMathInputRules(type: NodeType): InputRule[] {
  const makeRule = (find: typeof findSingleDollarMath) =>
    new InputRule({
      find,
      handler: ({ state, range, match }) => {
        const latex = (match.data as { latex?: string } | undefined)?.latex;
        if (!latex) return null;
        state.tr.replaceWith(range.from, range.to, type.create({ latex }));
      },
    });
  return [makeRule(findDoubleDollarInlineMath), makeRule(findSingleDollarMath)];
}

export function createBlockMathInputRules(
  type: NodeType,
  onEditMath?: (target: MathEditTarget) => void,
): InputRule[] {
  return [
    new InputRule({
      find: /^\$\$([^$\n]+)\$\$$/u,
      handler: ({ state, range, match }) => {
        const latex = (match[1] ?? "").trim();
        if (!latex) return null;
        const $from = state.doc.resolve(range.from);
        if ($from.depth < 1 || !$from.parent.isTextblock) return null;
        state.tr.replaceWith($from.before(), $from.after(), type.create({ latex }));
      },
    }),
    new InputRule({
      find: /^\$\$\s$/u,
      handler: ({ state, range }) => {
        const $from = state.doc.resolve(range.from);
        if ($from.depth < 1 || !$from.parent.isTextblock) return null;
        const insertPos = $from.before();
        state.tr.replaceWith(insertPos, $from.after(), type.create({ latex: "" }));
        queueMicrotask(() => onEditMath?.({ kind: "block", pos: insertPos, latex: "", createdEmpty: true }));
      },
    }),
  ];
}

interface InsertMathOptions {
  readonly initialLatex?: string;
  readonly range?: { from: number; to: number };
}

function insertBlockMathNode(
  editor: Editor,
  nodeType: NodeType,
  latex: string,
  range: { from: number; to: number },
) {
  const { state, view } = editor;
  const $from = state.doc.resolve(range.from);
  const replaceEmptyPara =
    $from.depth >= 1 && $from.parent.type.name === "paragraph" && $from.parent.content.size === 0;
  const insertPos = replaceEmptyPara ? $from.before() : range.from;
  const tr = replaceEmptyPara
    ? state.tr.replaceWith($from.before(), $from.after(), nodeType.create({ latex }))
    : state.tr.replaceRangeWith(range.from, range.to, nodeType.create({ latex }));
  view.dispatch(tr.scrollIntoView());
  return insertPos;
}

export function insertAndEditMath(
  editor: Editor,
  kind: "inline" | "block",
  onEditMath?: (target: MathEditTarget) => void,
  options: InsertMathOptions = {},
): boolean {
  const { state, view } = editor;
  const nodeType = state.schema.nodes[kind === "block" ? "blockMath" : "inlineMath"];
  if (!nodeType) return false;
  const range = options.range ?? { from: state.selection.from, to: state.selection.to };
  const latex = (options.initialLatex ?? "").trim();

  const pos =
    kind === "block"
      ? insertBlockMathNode(editor, nodeType, latex, range)
      : (view.dispatch(
          state.tr.replaceRangeWith(range.from, range.to, nodeType.create({ latex })).scrollIntoView(),
        ),
        range.from);
  onEditMath?.({ kind, pos, latex, createdEmpty: !latex });
  return true;
}

export function openSelectedMathNode(
  editor: Editor,
  kind: "block" | "inline",
  onEditMath?: (target: MathEditTarget) => void,
): boolean {
  const { selection } = editor.state;
  const nodeName = kind === "block" ? "blockMath" : "inlineMath";
  if (!(selection instanceof NodeSelection) || selection.node.type.name !== nodeName) return false;
  const latex = typeof selection.node.attrs.latex === "string" ? selection.node.attrs.latex : "";
  onEditMath?.({ kind, pos: selection.from, latex });
  return true;
}

export function handleBlockMathEnter(editor: Editor, onEditMath?: (target: MathEditTarget) => void): boolean {
  if (openSelectedMathNode(editor, "block", onEditMath)) return true;
  const { selection } = editor.state;
  const { $from } = selection;
  const isDollarLine =
    selection.empty &&
    $from.depth >= 1 &&
    $from.parent.type.name === "paragraph" &&
    $from.parent.textContent.trim() === "$$";
  const blockMath = isDollarLine ? editor.state.schema.nodes.blockMath : undefined;
  if (!blockMath) return false;
  const insertPos = $from.before();
  editor.view.dispatch(
    editor.state.tr.replaceWith(insertPos, $from.after(), blockMath.create({ latex: "" })),
  );
  onEditMath?.({ kind: "block", pos: insertPos, latex: "", createdEmpty: true });
  return true;
}

export function convertSelectionToInlineMath(
  editor: Editor,
  onEditMath?: (target: MathEditTarget) => void,
): boolean {
  if (!editor.isEditable || editor.isActive("codeBlock")) return false;
  const { from, to, empty } = editor.state.selection;
  const initialLatex = empty ? "" : editor.state.doc.textBetween(from, to, " ").trim();
  return insertAndEditMath(editor, "inline", onEditMath, { initialLatex, range: { from, to } });
}
