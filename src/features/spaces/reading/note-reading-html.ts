import { getHTMLFromFragment } from "@tiptap/core";
import { Node } from "@tiptap/pm/model";
import DOMPurify from "dompurify";

import type { TiptapJsonNode } from "../document-editing/tiptap-document";
import { createCanvasTextSchema } from "../document-editing/tiptap-extensions";

export interface ReaderOutlineHeading {
  index: number;
  level: number;
  text: string;
}

const noteSchema = createCanvasTextSchema("note");
const htmlCache = new WeakMap<TiptapJsonNode, string>();
const ALLOWED_READER_URI_REGEXP =
  /^(?:(?:(?:f|ht)tps?|asset|mailto|tel|callto|sms|cid|xmpp):|[^a-z]|[a-z+.-]+(?:[^a-z+.:-]|$))/i;
const ATX_HEADING = /^(#{1,4})\s+(.+?)\s*#*$/u;
const FENCE_TOGGLE = /^(?:```|~~~)/u;

/** Renders a Note's TipTap document into sanitized HTML using Knovum's shared note schema. */
export function noteReadingHtml(content: TiptapJsonNode): string {
  const cached = htmlCache.get(content);
  if (cached !== undefined) return cached;
  const rawHtml = getHTMLFromFragment(Node.fromJSON(noteSchema, content).content, noteSchema);
  const sanitized = DOMPurify.sanitize(rawHtml, {
    ALLOWED_URI_REGEXP: ALLOWED_READER_URI_REGEXP,
  });
  htmlCache.set(content, sanitized);
  return sanitized;
}

/** Extracts top-level ATX headings from Markdown source, skipping fenced code blocks. */
export function extractMarkdownHeadings(markdown: string): readonly ReaderOutlineHeading[] {
  const headings: ReaderOutlineHeading[] = [];
  let inFence = false;
  for (const line of markdown.split(/\r?\n/u)) {
    const trimmed = line.trim();
    if (FENCE_TOGGLE.test(trimmed)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    const match = ATX_HEADING.exec(trimmed);
    if (!match) continue;
    const text = match[2]!.trim();
    if (text) headings.push({ index: headings.length, level: match[1]!.length, text });
  }
  return headings;
}

function collectNodeText(node: TiptapJsonNode): string {
  if (typeof node.text === "string") return node.text;
  if (!Array.isArray(node.content)) return "";
  return node.content.map(collectNodeText).join("");
}

/** Extracts heading nodes from a TipTap JSON document for reader TOC navigation. */
export function extractTiptapHeadings(content: TiptapJsonNode): readonly ReaderOutlineHeading[] {
  const headings: ReaderOutlineHeading[] = [];
  const walk = (node: TiptapJsonNode) => {
    if (node.type === "heading") {
      const level = typeof node.attrs?.level === "number" ? node.attrs.level : 1;
      const text = collectNodeText(node).trim();
      if (text && level <= 4) headings.push({ index: headings.length, level, text });
      return;
    }
    if (Array.isArray(node.content)) node.content.forEach(walk);
  };
  walk(content);
  return headings;
}

/** Scrolls a rendered Markdown/Note prose container to a cited heading or quoted text snippet. */
export function scrollProseToLocator(container: HTMLElement | null, heading?: string, quote?: string): void {
  if (!container) return;
  const targetHeading = heading?.trim();
  if (targetHeading) {
    const match = [...container.querySelectorAll<HTMLElement>("h1, h2, h3, h4")].find(
      (h) => h.textContent?.trim() === targetHeading,
    );
    if (match) {
      match.scrollIntoView({ behavior: "smooth", block: "start" });
      return;
    }
  }
  const needle = quote?.trim();
  if (!needle) return;
  [...container.querySelectorAll<HTMLElement>("p, li, blockquote, pre, h1, h2, h3, h4")]
    .find((el) => el.textContent?.includes(needle))
    ?.scrollIntoView({ behavior: "smooth", block: "center" });
}
