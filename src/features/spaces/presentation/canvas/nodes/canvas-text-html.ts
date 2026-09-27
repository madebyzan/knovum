import { generateHTML } from "@tiptap/core";
import DOMPurify from "dompurify";

import type { TiptapJsonNode } from "../../../editor/tiptap-document";
import { createCanvasTextExtensions } from "../../../editor/tiptap-extensions";

const extensions = {
  note: createCanvasTextExtensions("", "note"),
  text: createCanvasTextExtensions("", "text"),
};
const htmlByContent = {
  note: new WeakMap<TiptapJsonNode, string>(),
  text: new WeakMap<TiptapJsonNode, string>(),
};

export function canvasTextHtml(content: TiptapJsonNode, kind: "note" | "text" = "text") {
  const cached = htmlByContent[kind].get(content);
  if (cached !== undefined) return cached;
  const html = DOMPurify.sanitize(generateHTML(content, extensions[kind]));
  htmlByContent[kind].set(content, html);
  return html;
}
