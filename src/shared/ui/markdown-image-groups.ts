import type { MarkdownIt, StateBlock, StateCore } from "markdown-it";

import { renderImageGroupHtml } from "./image-group-html";
import {
  normalizeImageGroupLayout,
  type ImageGroupLayout,
  type LightboxImageItem,
} from "./image-group-layout";
import { isDirectiveCloseLine, parseDirectiveOpenLine } from "./markdown-directives";
import { isVideoMediaUrl, parsePendingMediaDirective } from "./markdown-media";

function findDirectiveEndLine(
  state: StateBlock,
  startLine: number,
  endLine: number,
  fenceLength: number,
): number {
  let depth = 1;
  for (let lineIndex = startLine + 1; lineIndex < endLine; lineIndex += 1) {
    const lineStart = state.bMarks[lineIndex]! + state.tShift[lineIndex]!;
    const line = state.src.slice(lineStart, state.eMarks[lineIndex]!);
    if (parseDirectiveOpenLine(line)) depth += 1;
    else if (isDirectiveCloseLine(line, fenceLength)) {
      depth -= 1;
      if (depth === 0) return lineIndex;
    }
  }
  return -1;
}

function parseGalleryDirective(
  state: StateBlock,
  startLine: number,
  endLine: number,
  silent: boolean,
): boolean {
  const startPos = state.bMarks[startLine]! + state.tShift[startLine]!;
  const open = parseDirectiveOpenLine(state.src.slice(startPos, state.eMarks[startLine]!));
  if (!open || open.name !== "gallery") return false;

  const closeLine = findDirectiveEndLine(state, startLine, endLine, open.fenceLength);
  if (closeLine === -1) return false;
  if (silent) return true;

  const layout = normalizeImageGroupLayout(open.attrs.layout ?? "gallery");
  const openToken = state.push("image_group_open", "div", 1);
  openToken.meta = { layout };
  openToken.map = [startLine, closeLine + 1];

  const oldParent = state.parentType;
  const oldLineMax = state.lineMax;
  state.lineMax = closeLine;
  state.md.block.tokenize(state, startLine + 1, closeLine);
  state.parentType = oldParent;
  state.lineMax = oldLineMax;

  state.push("image_group_close", "div", -1);
  state.line = closeLine + 1;
  return true;
}

type InlineChild = NonNullable<StateCore["tokens"][number]["children"]>[number];

function isIgnorableChild(child: InlineChild): boolean {
  if (child.type === "softbreak" || child.type === "hardbreak") return true;
  return child.type === "text" && !child.content.trim();
}

function isParagraphTriple(tokens: StateCore["tokens"], openIndex: number): boolean {
  return (
    tokens[openIndex]?.type === "paragraph_open" &&
    tokens[openIndex + 1]?.type === "inline" &&
    tokens[openIndex + 2]?.type === "paragraph_close"
  );
}

function toImageItem(child: InlineChild): LightboxImageItem | null {
  const raw = child.attrGet("src");
  const src = raw === null ? "" : String(raw);
  if (!src || isVideoMediaUrl(src) || parsePendingMediaDirective(src)) return null;
  return { src, caption: child.content ?? "" };
}

function extractImagesFromParagraph(
  tokens: StateCore["tokens"],
  openIndex: number,
): { images: LightboxImageItem[]; nextIndex: number } | null {
  if (!isParagraphTriple(tokens, openIndex)) return null;
  const images: LightboxImageItem[] = [];
  for (const child of tokens[openIndex + 1]!.children ?? []) {
    if (child.type === "image") {
      const item = toImageItem(child);
      if (!item) return null;
      images.push(item);
    } else if (!isIgnorableChild(child)) {
      return null;
    }
  }
  return images.length ? { images, nextIndex: openIndex + 3 } : null;
}

function collectImagesInRange(tokens: StateCore["tokens"], start: number, end: number): LightboxImageItem[] {
  const images: LightboxImageItem[] = [];
  for (let i = start; i < end; i += 1) {
    const token = tokens[i]!;
    if (token.type !== "inline" || !token.children) continue;
    for (const child of token.children) {
      if (child.type !== "image") continue;
      const item = toImageItem(child);
      if (item) images.push(item);
    }
  }
  return images;
}

function makeGroupToken(
  state: StateCore,
  layout: ImageGroupLayout,
  images: readonly LightboxImageItem[],
  map?: [number, number] | null,
) {
  const token = new state.Token("image_group_rendered", "div", 0);
  token.meta = { layout, images };
  if (map) token.map = map;
  return token;
}

function consumeExplicitGroup(state: StateCore, openIndex: number, output: StateCore["tokens"]): number {
  const source = state.tokens;
  let closeIdx = openIndex + 1;
  while (closeIdx < source.length && source[closeIdx]!.type !== "image_group_close") closeIdx += 1;
  const meta = source[openIndex]?.meta as { layout?: unknown } | undefined;
  const images = collectImagesInRange(source, openIndex + 1, closeIdx);
  if (images.length) {
    output.push(
      makeGroupToken(state, normalizeImageGroupLayout(meta?.layout), images, source[openIndex]?.map),
    );
  }
  return closeIdx + 1;
}

function mergeTokenMap(
  firstMap: [number, number] | null | undefined,
  lastMap: [number, number] | null | undefined,
): [number, number] | null {
  return firstMap && lastMap ? [firstMap[0], lastMap[1]] : null;
}

function consumeParagraphRun(state: StateCore, startIndex: number, output: StateCore["tokens"]): number {
  const source = state.tokens;
  const firstPara = extractImagesFromParagraph(source, startIndex);
  if (!firstPara) {
    output.push(source[startIndex]!);
    return startIndex + 1;
  }
  const grouped: LightboxImageItem[] = [...firstPara.images];
  let cursor = firstPara.nextIndex;
  let lastMap = source[startIndex]?.map;
  while (cursor < source.length) {
    const nextPara = extractImagesFromParagraph(source, cursor);
    if (!nextPara) break;
    grouped.push(...nextPara.images);
    lastMap = source[cursor]?.map ?? lastMap;
    cursor = nextPara.nextIndex;
  }
  if (grouped.length >= 2) {
    output.push(makeGroupToken(state, "auto", grouped, mergeTokenMap(source[startIndex]?.map, lastMap)));
    return cursor;
  }
  output.push(source[startIndex]!, source[startIndex + 1]!, source[startIndex + 2]!);
  return firstPara.nextIndex;
}

function transformImageGroups(state: StateCore) {
  const output: StateCore["tokens"] = [];
  let i = 0;
  while (i < state.tokens.length) {
    i =
      state.tokens[i]!.type === "image_group_open"
        ? consumeExplicitGroup(state, i, output)
        : consumeParagraphRun(state, i, output);
  }
  state.tokens = output;
}

export function registerImageGroupMarkdownPlugin(md: MarkdownIt): void {
  md.block.ruler.before("fence", "image_group_directive", parseGalleryDirective, {
    alt: ["paragraph", "reference", "blockquote", "list"],
  });
  md.core.ruler.after("gfm_task_lists", "auto_image_groups", transformImageGroups);
  md.renderer.rules.image_group_rendered = (tokens, index) => {
    const meta = tokens[index]?.meta as
      | { layout?: ImageGroupLayout; images?: readonly LightboxImageItem[] }
      | undefined;
    if (!meta?.images?.length) return "";
    return renderImageGroupHtml(normalizeImageGroupLayout(meta.layout), meta.images);
  };
}
