import { getHTMLFromFragment, getSchema } from "@tiptap/core";
import { Node } from "@tiptap/pm/model";
import DOMPurify from "dompurify";

import type { TiptapJsonNode } from "../../../document-editing/tiptap-document";
import { createCanvasTextExtensions } from "../../../document-editing/tiptap-extensions";

/** One schema per text kind: building it is most of the cost of rendering a preview. */
const schemas = {
  note: getSchema(createCanvasTextExtensions("", "note")),
  text: getSchema(createCanvasTextExtensions("", "text")),
};
const htmlByContent = {
  note: new WeakMap<TiptapJsonNode, string>(),
  text: new WeakMap<TiptapJsonNode, string>(),
};

export function canvasTextHtml(content: TiptapJsonNode, kind: "note" | "text" = "text") {
  const cached = htmlByContent[kind].get(content);
  if (cached !== undefined) return cached;
  const schema = schemas[kind];
  const html = DOMPurify.sanitize(getHTMLFromFragment(Node.fromJSON(schema, content).content, schema));
  htmlByContent[kind].set(content, html);
  return html;
}
