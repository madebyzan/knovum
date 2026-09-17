import type { TiptapJsonNode } from "./tiptap-document";

export function emptyTiptapContent(): TiptapJsonNode {
  return { type: "doc", content: [{ type: "paragraph" }] };
}

export function plainTextTiptapContent(text: string): TiptapJsonNode {
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
