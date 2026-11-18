import { getHTMLFromFragment } from "@tiptap/core";
import { Node } from "@tiptap/pm/model";
import DOMPurify from "dompurify";

import type { TiptapJsonNode } from "../../../document-editing/tiptap-document";
import { createCanvasTextSchema } from "../../../document-editing/tiptap-extensions";

/** One schema per text kind: building it is most of the cost of rendering a preview. */
const schemas = {
  note: createCanvasTextSchema("note"),
  text: createCanvasTextSchema("text"),
};
const htmlByContent = {
  note: new WeakMap<TiptapJsonNode, string>(),
  text: new WeakMap<TiptapJsonNode, string>(),
};
const ALLOWED_CANVAS_URI_REGEXP =
  /^(?:(?:(?:f|ht)tps?|asset|mailto|tel|callto|sms|cid|xmpp):|[^a-z]|[a-z+.-]+(?:[^a-z+.:-]|$))/i;

export function canvasTextHtml(content: TiptapJsonNode, kind: "note" | "text" = "text") {
  const cached = htmlByContent[kind].get(content);
  if (cached !== undefined) return cached;
  const schema = schemas[kind];
  const html = DOMPurify.sanitize(getHTMLFromFragment(Node.fromJSON(schema, content).content, schema), {
    ALLOWED_URI_REGEXP: ALLOWED_CANVAS_URI_REGEXP,
  });
  htmlByContent[kind].set(content, html);
  return html;
}
