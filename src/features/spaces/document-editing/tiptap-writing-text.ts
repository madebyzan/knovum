import type { Editor, JSONContent } from "@tiptap/vue-3";

import type { EditorTransformAction } from "../../../shared/contracts/generated/EditorTransformAction";
import { serializeRangeAsMarkdown } from "./tiptap-ai-context";
import { setReasoningTraceRange } from "./tiptap-reasoning-trace";

export interface TransformSource {
  readonly text: string;
  readonly from: number;
  readonly to: number;
  readonly hasSelection: boolean;
}

const OUTER_MARKDOWN_FENCE = /^```(?:markdown|md)[ \t]*\r?\n([\s\S]*?)\r?\n```$/iu;
const LEADING_H1_PATTERN = /^#[ \t]+([^\r\n]+?)(?:[ \t]+#+)?[ \t]*(?:\r?\n|$)([\s\S]*)$/u;

export function normalizeGeneratedMarkdown(generated: string): string {
  const trimmed = generated.trim();
  const match = OUTER_MARKDOWN_FENCE.exec(trimmed);
  return match?.[1] ? match[1].trim() : trimmed;
}

/**
 * Splits a leading `# Title` heading from a generated Markdown article so an empty untitled
 * document can promote it into the document title instead of duplicating an H1 in the body.
 */
function extractLeadingHeadingTitle(
  generated: string,
): { readonly title: string; readonly body: string } | undefined {
  const normalized = normalizeGeneratedMarkdown(generated);
  const match = LEADING_H1_PATTERN.exec(normalized);
  const title = match?.[1]?.trim();
  const body = match?.[2]?.trim();
  return title && body ? { title, body } : undefined;
}

export interface WritingToolsOptions {
  readonly documentTitle?: { readonly value: string } | undefined;
  readonly promoteTitle?: ((title: string) => boolean) | undefined;
}

export function acceptGeneratedDraft(
  editor: Editor,
  input: {
    readonly action: EditorTransformAction;
    readonly source: TransformSource;
    readonly draft: string;
    readonly promoteTitle?: ((title: string) => boolean) | undefined;
  },
): boolean {
  let content = input.draft;
  if (!input.source.hasSelection && !editor.getText().trim() && input.promoteTitle) {
    const extracted = extractLeadingHeadingTitle(input.draft);
    if (extracted && input.promoteTitle(extracted.title)) content = extracted.body;
  }
  return applyGeneratedText(editor, input.action, input.source, content);
}

export function createUntitledTitlePromoter(
  title: { value: string },
  getPlaceholder: () => string,
): (heading: string) => boolean {
  return (heading: string) => {
    const current = title.value.trim();
    if (current && current !== getPlaceholder().trim()) return false;
    title.value = heading;
    return true;
  };
}

export function markReasoning(editor: Editor, source: TransformSource) {
  if (!source.hasSelection || source.to <= source.from) return;
  setReasoningTraceRange(editor.view, source.from, source.to);
}

export function captureSelectionOrCursor(
  editor: Editor,
  explicitRange?: { from: number; to: number },
): TransformSource {
  const { doc, selection } = editor.state;
  const rawFrom = explicitRange?.from ?? selection.from;
  const rawTo = explicitRange?.to ?? selection.to;
  const from = Math.max(0, Math.min(rawFrom, doc.content.size));
  const to = Math.max(from, Math.min(rawTo, doc.content.size));
  const hasSelection = to > from;
  const text = hasSelection ? serializeRangeAsMarkdown(editor, from, to) : "";
  return { from, to, text, hasSelection };
}

export function actionSource(
  editor: Editor,
  action: EditorTransformAction,
  explicitRange?: { from: number; to: number },
): TransformSource | undefined {
  const captured = captureSelectionOrCursor(editor, explicitRange);
  if (action === "custom") return captured;
  if (action === "complete") {
    if (captured.hasSelection) return captured;
    const preceding = editor.state.doc.textBetween(0, captured.from, "\n\n", "\n").trim();
    return preceding ? captured : undefined;
  }
  return captured.hasSelection && captured.text.trim() ? captured : undefined;
}

type ParsedMarkdown =
  | { readonly kind: "inline"; readonly content: JSONContent[] }
  | { readonly kind: "blocks"; readonly content: JSONContent[] };

function extractInlineParagraph(nodes: JSONContent[]): JSONContent[] | undefined {
  if (nodes.length !== 1) return undefined;
  const sole = nodes[0];
  return sole && sole.type === "paragraph" && sole.content?.length ? sole.content : undefined;
}

function parseMarkdownPayload(editor: Editor, normalized: string): ParsedMarkdown | undefined {
  if (!editor.markdown) return undefined;
  const parsed = editor.markdown.parse(normalized);
  if (parsed.type !== "doc" || !parsed.content?.length) return undefined;
  const inline = extractInlineParagraph(parsed.content);
  return inline ? { kind: "inline", content: inline } : { kind: "blocks", content: parsed.content };
}

function insertStructuredAt(
  editor: Editor,
  target: number | { from: number; to: number },
  normalized: string,
  parsed: ParsedMarkdown | undefined,
): boolean {
  if (parsed) return editor.chain().focus().insertContentAt(target, parsed.content).run();
  return editor.chain().focus().insertContentAt(target, normalized, { contentType: "markdown" }).run();
}

function resolveInsertionTarget(editor: Editor, source: TransformSource, parsed: ParsedMarkdown | undefined) {
  const { doc } = editor.state;
  const pos = Math.max(0, Math.min(source.to, doc.content.size));
  if (parsed?.kind !== "blocks") return pos;
  const resolved = doc.resolve(pos);
  if (!source.hasSelection && resolved.depth === 1 && resolved.parent.type.name === "paragraph") {
    if (resolved.parent.content.size === 0) return { from: resolved.before(1), to: resolved.after(1) };
  }
  if (resolved.depth >= 1 && resolved.parentOffset === resolved.parent.content.size) {
    return Math.min(resolved.after(1), doc.content.size);
  }
  return pos;
}

function applyGeneratedText(
  editor: Editor,
  action: EditorTransformAction,
  source: TransformSource,
  generated: string,
): boolean {
  const normalized = normalizeGeneratedMarkdown(generated);
  if (!normalized) return false;
  const parsed = parseMarkdownPayload(editor, normalized);
  if (!source.hasSelection || action === "complete") {
    const target = resolveInsertionTarget(editor, source, parsed);
    return insertStructuredAt(editor, target, normalized, parsed);
  }
  const { doc } = editor.state;
  const sameBlock = doc.resolve(source.from).sameParent(doc.resolve(source.to));
  if (sameBlock && parsed?.kind === "inline") {
    return editor.chain().focus().insertContentAt({ from: source.from, to: source.to }, parsed.content).run();
  }
  return insertStructuredAt(editor, { from: source.from, to: source.to }, normalized, parsed);
}

export function insertGeneratedTextBelow(
  editor: Editor,
  source: TransformSource,
  generated: string,
): boolean {
  const normalized = normalizeGeneratedMarkdown(generated);
  if (!normalized) return false;
  const doc = editor.state.doc;
  const clamped = Math.max(0, Math.min(source.to, doc.content.size));
  const resolved = doc.resolve(clamped);
  const insertPos = Math.min(resolved.depth >= 1 ? resolved.after(1) : clamped, doc.content.size);
  return insertStructuredAt(editor, insertPos, normalized, parseMarkdownPayload(editor, normalized));
}
