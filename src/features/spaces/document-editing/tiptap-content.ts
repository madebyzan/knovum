import type { TiptapJsonNode } from "./tiptap-document";

function plainTextTiptapContent(text: string): TiptapJsonNode {
  const lines = text.split(/\r?\n/u);
  return {
    type: "doc",
    content: lines.map((line) =>
      line ? { type: "paragraph", content: [{ type: "text", text: line }] } : { type: "paragraph" },
    ),
  };
}

export function createSpaceNotePayload(title: string, text = "") {
  return { kind: "note" as const, title, tiptapContent: plainTextTiptapContent(text), plainText: text };
}

export function createSpaceTextPayload(title: string, text = "") {
  return { kind: "text" as const, title, tiptapContent: plainTextTiptapContent(text), plainText: text };
}

const isBlankParagraph = (node: TiptapJsonNode) => node.type === "paragraph" && !node.content?.length;

/**
 * `payload` with `text` added as paragraphs below a blank one. Its `plainText` is what the editor
 * would save for the result: blocks joined by line breaks, the blank paragraph an empty line.
 */
export function appendPlainText<T extends { tiptapContent: TiptapJsonNode; plainText: string }>(
  payload: T,
  text: string,
): T {
  const content = payload.tiptapContent.content ?? [];
  const kept = content.slice(0, content.findLastIndex((node) => !isBlankParagraph(node)) + 1);
  const added = plainTextTiptapContent(text).content ?? [];
  const before = payload.plainText.replace(/\n+$/u, "");
  return {
    ...payload,
    tiptapContent: {
      ...payload.tiptapContent,
      content: kept.length ? [...kept, { type: "paragraph" }, ...added] : added,
    },
    plainText: kept.length ? `${before}\n\n${text}` : text,
  };
}
