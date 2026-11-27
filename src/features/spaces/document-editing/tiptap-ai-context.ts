import type { Editor } from "@tiptap/vue-3";

import type { EditorContextReference } from "../../../shared/contracts/generated/EditorContextReference";
import type { EditorDocumentContext } from "../../../shared/contracts/generated/EditorDocumentContext";

const MAX_PRECEDING_CHARS = 1400;
const MAX_FOLLOWING_CHARS = 700;
const MAX_REFERENCES = 5;
const MAX_REFERENCE_EXCERPT = 1200;

function trimTail(value: string, maxChars: number): string | undefined {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  return trimmed.length <= maxChars ? trimmed : trimmed.slice(trimmed.length - maxChars).trimStart();
}

function trimHead(value: string, maxChars: number): string | undefined {
  const trimmed = value.trim();
  if (!trimmed) return undefined;
  return trimmed.length <= maxChars ? trimmed : trimmed.slice(0, maxChars).trimEnd();
}

function findEnclosingSectionHeading(editor: Editor, pos: number): string | undefined {
  const stack: Array<{ level: number; text: string }> = [];
  editor.state.doc.descendants((node, nodePos) => {
    if (nodePos >= pos) return false;
    if (node.type.name !== "heading") return true;
    const level = Number(node.attrs.level ?? 1);
    const text = node.textContent.trim();
    if (!text) return false;
    while (stack.length && stack[stack.length - 1]!.level >= level) stack.pop();
    stack.push({ level, text });
    return false;
  });
  return stack.length ? stack.map((item) => item.text).join(" > ") : undefined;
}

function normalizeReferences(references?: readonly EditorContextReference[]): EditorContextReference[] {
  if (!references?.length) return [];
  const normalized: EditorContextReference[] = [];
  for (const item of references.slice(0, MAX_REFERENCES)) {
    const excerpt = trimHead(item.excerpt, MAX_REFERENCE_EXCERPT);
    if (excerpt) normalized.push({ nodeId: item.nodeId, title: item.title.trim(), excerpt });
  }
  return normalized;
}

export function serializeRangeAsMarkdown(editor: Editor, from: number, to: number): string {
  const { doc } = editor.state;
  const clampedFrom = Math.max(0, Math.min(from, doc.content.size));
  const clampedTo = Math.max(clampedFrom, Math.min(to, doc.content.size));
  if (clampedTo <= clampedFrom) return "";
  const markdownManager = editor.markdown;
  if (!markdownManager) return doc.textBetween(clampedFrom, clampedTo, "\n\n", "\n");
  const slice = doc.slice(clampedFrom, clampedTo);
  const json = slice.toJSON() as { content?: Array<Record<string, unknown>> } | null;
  const nodes = json?.content ?? [];
  if (!nodes.length) return "";
  const docJson = slice.content.firstChild?.isInline
    ? { type: "doc", content: [{ type: "paragraph", content: nodes }] }
    : { type: "doc", content: nodes };
  const serialized = markdownManager.serialize(docJson).trim();
  return serialized || doc.textBetween(clampedFrom, clampedTo, "\n\n", "\n");
}

export interface ExtractEditorContextOptions {
  readonly documentTitle?: string | undefined;
  readonly references?: readonly EditorContextReference[] | undefined;
}

export function extractEditorDocumentContext(
  editor: Editor,
  range: { from: number; to: number },
  options: ExtractEditorContextOptions = {},
): EditorDocumentContext {
  const doc = editor.state.doc;
  const from = Math.max(0, Math.min(range.from, doc.content.size));
  const to = Math.max(from, Math.min(range.to, doc.content.size));
  const context: EditorDocumentContext = {};
  const documentTitle = options.documentTitle?.trim();
  const sectionHeading = findEnclosingSectionHeading(editor, from);
  const precedingText =
    from > 0 ? trimTail(serializeRangeAsMarkdown(editor, 0, from), MAX_PRECEDING_CHARS) : undefined;
  const followingText =
    to < doc.content.size
      ? trimHead(serializeRangeAsMarkdown(editor, to, doc.content.size), MAX_FOLLOWING_CHARS)
      : undefined;
  const references = normalizeReferences(options.references);

  if (documentTitle) context.documentTitle = documentTitle;
  if (sectionHeading) context.sectionHeading = sectionHeading;
  if (precedingText) context.precedingText = precedingText;
  if (followingText) context.followingText = followingText;
  if (references.length) context.references = references;
  return context;
}
