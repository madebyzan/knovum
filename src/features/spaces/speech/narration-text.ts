import MarkdownIt from "markdown-it";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { nodeSpeechText } from "./node-speech";
import { prepareSpeechMarkdown, pruneNonNarrativeDom, sanitizeSpokenText } from "./speech-text-sanitizer";

export function narrationText(node: SpaceCanvasNode): string {
  const text = nodeSpeechText(node);
  if (node.data.kind !== "markdown") return sanitizeSpokenText(text);
  const markup = new MarkdownIt({ html: false, breaks: true }).render(prepareSpeechMarkdown(text));
  const document = new DOMParser().parseFromString(markup, "text/html");
  pruneNonNarrativeDom(document);
  return sanitizeSpokenText(document.body.textContent ?? "");
}
