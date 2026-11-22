import { Extension, type Editor, type JSONContent } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";

import { TIPTAP_IMAGE_MIME_TYPES } from "./tiptap-images";
import { normalizeUrl } from "./url-validation";

const STRUCTURAL_HTML_SELECTOR = "h1, h2, h3, h4, h5, h6, ul, ol, blockquote, img, hr";
const INLINE_HTML_SELECTOR = "strong, b, em, i, a[href], s, del, u, mark, sub, sup";
const SLICE_RICH_SELECTOR = `${STRUCTURAL_HTML_SELECTOR}, ${INLINE_HTML_SELECTOR}, table, pre, code, button, [data-type], [data-citation-index]`;
const CODE_TABLE_MARKER_SELECTOR = "[data-line-number], [data-code-marker], .blob-code, .blob-num";
const MARKDOWN_EDITOR_MODES = new Set(["markdown", "mdx", "plaintext"]);
const SAFE_LINK_PROTOCOLS = ["http:", "https:", "mailto:"] as const;
const SAFE_IMAGE_PROTOCOLS = ["http:", "https:"] as const;
const BARE_URL_PATTERN = /^https?:\/\/\S+$/iu;

type HtmlRichness = "none" | "inline" | "structural";

function hasSemanticTable(body: HTMLElement): boolean {
  return Array.from(body.querySelectorAll("table")).some(
    (table) => !table.querySelector(CODE_TABLE_MARKER_SELECTOR),
  );
}

function classifyClipboardHtml(html: string): HtmlRichness {
  if (!html.trim()) return "none";
  const body = new DOMParser().parseFromString(html, "text/html").body;
  if (html.includes("data-pm-slice")) {
    return body.querySelector(SLICE_RICH_SELECTOR) ? "structural" : "none";
  }
  if (body.querySelector(STRUCTURAL_HTML_SELECTOR) || hasSemanticTable(body)) return "structural";
  return body.querySelector(INLINE_HTML_SELECTOR) ? "inline" : "none";
}

function normalizeClipboardMarkdown(plain: string): string {
  return plain
    .replace(/\uFEFF/gu, "")
    .replace(/\r\n?|\u2028|\u2029/gu, "\n")
    .replace(/\u00A0/gu, " ");
}

function isNonMarkdownCodeEditorCopy(clipboard: DataTransfer): boolean {
  const raw = clipboard.getData("vscode-editor-data");
  if (!raw) return false;
  try {
    const mode = (JSON.parse(raw) as { mode?: unknown }).mode;
    return typeof mode === "string" && mode.length > 0 && !MARKDOWN_EDITOR_MODES.has(mode);
  } catch {
    return false;
  }
}

function hasImageAttachment(clipboard: DataTransfer): boolean {
  return Array.from(clipboard.files).some((file) => TIPTAP_IMAGE_MIME_TYPES.includes(file.type));
}

function nodeHasBlockMarkdown(node: JSONContent): boolean {
  if (node.type && node.type !== "doc" && node.type !== "paragraph" && node.type !== "text") {
    return true;
  }
  return node.content?.some(nodeHasBlockMarkdown) ?? false;
}

function nodeHasInlineMarkdown(node: JSONContent): boolean {
  if (node.marks?.length) return true;
  return node.content?.some(nodeHasInlineMarkdown) ?? false;
}

function sanitizeMarks(rawMarks: NonNullable<JSONContent["marks"]>): NonNullable<JSONContent["marks"]> {
  const sanitized: NonNullable<JSONContent["marks"]> = [];
  for (const mark of rawMarks) {
    if (mark.type !== "link") {
      sanitized.push(mark);
      continue;
    }
    const href =
      typeof mark.attrs?.href === "string" ? normalizeUrl(mark.attrs.href, SAFE_LINK_PROTOCOLS) : undefined;
    if (!href) continue;
    sanitized.push({
      type: mark.type,
      attrs: Object.assign({}, mark.attrs, { href }),
    });
  }
  return sanitized;
}

function sanitizeImageNode(node: JSONContent): JSONContent | null {
  const src =
    typeof node.attrs?.src === "string" ? normalizeUrl(node.attrs.src, SAFE_IMAGE_PROTOCOLS) : undefined;
  return src ? { ...node, attrs: Object.assign({}, node.attrs, { src }) } : null;
}

function sanitizeChildren(rawContent: readonly JSONContent[]): JSONContent[] {
  const children: JSONContent[] = [];
  for (const child of rawContent) {
    const next = sanitizeNode(child);
    if (next) children.push(next);
  }
  return children;
}

function sanitizeNode(node: JSONContent): JSONContent | null {
  if (node.type === "image") return sanitizeImageNode(node);
  const { marks: rawMarks, content: rawContent, ...rest } = node;
  const marks = rawMarks ? sanitizeMarks(rawMarks) : [];
  return {
    ...rest,
    ...(marks.length ? { marks } : {}),
    ...(rawContent ? { content: sanitizeChildren(rawContent) } : {}),
  };
}

interface InsertableMarkdown {
  readonly kind: "inline" | "blocks";
  readonly content: JSONContent[];
}

function extractInsertableContent(parsed: JSONContent): InsertableMarkdown | null {
  const sanitized = sanitizeNode(parsed);
  const blocks = sanitized?.content ?? [];
  if (!blocks.length) return null;
  if (blocks.length === 1 && blocks[0]?.type === "paragraph" && blocks[0].content?.length) {
    return { kind: "inline", content: blocks[0].content };
  }
  return { kind: "blocks", content: blocks };
}

function insertParsedMarkdown(editor: Editor, view: EditorView, insertable: InsertableMarkdown): boolean {
  const { selection } = view.state;
  const { $from } = selection;
  const isEmptyParagraph =
    insertable.kind === "blocks" &&
    selection.empty &&
    $from.parent.type.name === "paragraph" &&
    $from.parent.content.size === 0;
  if (isEmptyParagraph) {
    return editor.commands.insertContentAt({ from: $from.before(), to: $from.after() }, insertable.content);
  }
  return editor.commands.insertContent(insertable.content);
}

function canAttemptMarkdownPaste(editor: Editor, view: EditorView, clipboard: DataTransfer | null): boolean {
  if (!clipboard || !editor.markdown) return false;
  if (view.state.selection.$from.parent.type.spec.code) return false;
  if (hasImageAttachment(clipboard) || isNonMarkdownCodeEditorCopy(clipboard)) return false;
  const trimmed = clipboard.getData("text/plain").trim();
  return Boolean(trimmed) && !BARE_URL_PATTERN.test(trimmed);
}

function handleMarkdownPaste(editor: Editor, view: EditorView, event: ClipboardEvent): boolean {
  const clipboard = event.clipboardData;
  if (!canAttemptMarkdownPaste(editor, view, clipboard)) return false;

  const htmlRichness = classifyClipboardHtml(clipboard!.getData("text/html"));
  if (htmlRichness === "structural") return false;

  const plain = normalizeClipboardMarkdown(clipboard!.getData("text/plain"));
  const parsed = editor.markdown!.parse(plain);
  const hasBlock = nodeHasBlockMarkdown(parsed);
  if (!hasBlock && (htmlRichness === "inline" || !nodeHasInlineMarkdown(parsed))) return false;

  const insertable = extractInsertableContent(parsed);
  if (!insertable) return false;
  event.preventDefault();
  return insertParsedMarkdown(editor, view, insertable);
}

/**
 * Parses pasted plain-text (or syntax-highlighted source) Markdown through the active
 * `@tiptap/markdown` schema while preserving internal ProseMirror slices, structural rich HTML,
 * non-Markdown VS Code language snippets, image pastes, bare-URL linking, and code-block pastes.
 */
export const MarkdownPaste = Extension.create({
  name: "markdownPaste",
  priority: 1000,
  addProseMirrorPlugins() {
    const editor = this.editor;
    return [
      new Plugin({
        key: new PluginKey("markdownPaste"),
        props: {
          handlePaste: (view, event) => handleMarkdownPaste(editor, view, event),
        },
      }),
    ];
  },
});
