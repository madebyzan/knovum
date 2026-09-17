import { generateHTML } from "@tiptap/core";
import DOMPurify from "dompurify";

import type { TiptapJsonNode } from "../../../editor/tiptap-document";
import { createCanvasTextExtensions } from "../../../editor/tiptap-extensions";

const extensions = createCanvasTextExtensions("");
const htmlByContent = new WeakMap<TiptapJsonNode, string>();

export function canvasTextHtml(content: TiptapJsonNode) {
  const cached = htmlByContent.get(content);
  if (cached !== undefined) return cached;
  const html = DOMPurify.sanitize(generateHTML(content, extensions));
  htmlByContent.set(content, html);
  return html;
}
