import { BlockMath, InlineMath } from "@tiptap/extension-mathematics";
import type { Node as ProseMirrorNode } from "@tiptap/pm/model";

import {
  isDigitChar,
  isEscapedAt,
  KATEX_BASE_OPTIONS,
  matchInlineMathAtStart,
} from "../../../shared/ui/math-rendering";

const BLOCK_MATH_PATTERN = /^\$\$(?!\$)(?:\r?\n([\s\S]+?)\r?\n|[ \t]*([^\r\n]+?)[ \t]*)\$\$(?!\$)/;

function findInlineMathStart(src: string): number {
  for (let index = 0; index < src.length; index += 1) {
    if (src.charCodeAt(index) !== 0x24) continue;
    if (src.charCodeAt(index + 1) === 0x24) {
      while (src.charCodeAt(index + 1) === 0x24) index += 1;
      continue;
    }
    if (isEscapedAt(src, index) || isDigitChar(src[index - 1])) continue;
    return index;
  }
  return -1;
}

const SpaceInlineMath = InlineMath.extend({
  renderText({ node }) {
    return String(node.attrs.latex ?? "");
  },
  markdownTokenizer: {
    name: "inlineMath",
    level: "inline" as const,
    start: findInlineMathStart,
    tokenize(src: string) {
      const matched = matchInlineMathAtStart(src);
      if (!matched) return undefined;
      return {
        type: "inlineMath",
        raw: matched.raw,
        latex: matched.latex,
      };
    },
  },
});

const SpaceBlockMath = BlockMath.extend({
  renderText({ node }) {
    return String(node.attrs.latex ?? "");
  },
  markdownTokenizer: {
    name: "blockMath",
    level: "block" as const,
    start(src: string) {
      return src.indexOf("$$");
    },
    tokenize(src: string) {
      const match = BLOCK_MATH_PATTERN.exec(src);
      if (!match) return undefined;
      const latex = (match[1] ?? match[2] ?? "").trim();
      if (!latex) return undefined;
      return {
        type: "blockMath",
        raw: match[0],
        latex,
      };
    },
  },
});

export function createSpaceMathExtensions() {
  return [
    SpaceInlineMath.configure({
      katexOptions: { ...KATEX_BASE_OPTIONS, displayMode: false },
    }),
    SpaceBlockMath.configure({
      katexOptions: { ...KATEX_BASE_OPTIONS, displayMode: true },
    }),
  ];
}

export interface PlainTextReadableDoc {
  content: { size: number };
  textBetween: (
    from: number,
    to: number,
    blockSeparator?: string,
    leafText?: string | ((leafNode: ProseMirrorNode) => string),
  ) => string;
}

function leafNodePlainText(node: ProseMirrorNode): string {
  if (node.type.name === "inlineMath" || node.type.name === "blockMath") {
    return String(node.attrs.latex ?? "");
  }
  return "\n";
}

/** Extracts plain text from a ProseMirror document while preserving math node LaTeX formulas. */
export function tiptapPlainText(doc: PlainTextReadableDoc, from = 0, to = doc.content.size): string {
  return doc.textBetween(from, to, "\n", leafNodePlainText);
}
