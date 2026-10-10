import MarkdownIt from "markdown-it";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { nodeSpeechText } from "./node-speech";

export function narrationText(node: SpaceCanvasNode): string {
  const text = nodeSpeechText(node);
  if (node.data.kind !== "markdown") return text;
  const markup = new MarkdownIt({ html: false, breaks: true }).render(text);
  const document = new DOMParser().parseFromString(markup, "text/html");
  document.querySelectorAll("img").forEach((element) => element.replaceWith(element.alt));
  return document.body.textContent?.trim() ?? "";
}
