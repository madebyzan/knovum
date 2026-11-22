import type { KnowledgeLocation } from "../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../knowledge/model/schema/persisted-chunk";
import { extractNormalizedPdfRects } from "./pdf-rect-geometry";

const BLOCK_TAGS = new Set([
  "P",
  "LI",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "BLOCKQUOTE",
  "PRE",
  "TR",
  "DIV",
  "SECTION",
  "ARTICLE",
]);

function closestElement(node: Node, selector: string): HTMLElement | undefined {
  const element = node instanceof Element ? node : node.parentElement;
  return element?.closest<HTMLElement>(selector) ?? undefined;
}

export function rangeCoversPoint(range: Range | undefined, point: { x: number; y: number }): boolean {
  return [...(range?.getClientRects() ?? [])].some(
    (box) => point.x >= box.left && point.x <= box.right && point.y >= box.top && point.y <= box.bottom,
  );
}

/** Normalizes selected DOM Range text while preserving paragraph boundaries and stripping UI chrome. */
export function extractReaderRangeText(range: Range): string {
  const fragment = range.cloneContents();
  for (const badge of fragment.querySelectorAll(".ui-citation, [data-reader-ignore-selection]")) {
    const before = badge.previousSibling;
    if (before?.nodeType === Node.TEXT_NODE) before.textContent = before.textContent!.replace(/[ \t]+$/u, "");
    badge.remove();
  }
  const lines: string[] = [""];
  const walk = (node: Node) => {
    if (node.nodeType === Node.TEXT_NODE) lines[lines.length - 1] += node.textContent ?? "";
    else if (node instanceof HTMLBRElement) lines.push("");
    else if (node instanceof HTMLTableCellElement && node.previousElementSibling)
      lines[lines.length - 1] += "\t";
    const isBlock = node instanceof Element && BLOCK_TAGS.has(node.tagName);
    if (isBlock) lines.push("");
    node.childNodes.forEach(walk);
    if (isBlock) lines.push("");
  };
  fragment.childNodes.forEach(walk);
  return lines
    .map((line) => line.replace(/[ \t]+$/u, "").replace(/ {2,}/gu, " "))
    .join("\n")
    .replace(/\n{3,}/gu, "\n\n")
    .trim();
}

function findClosestInRange(range: Range, selector: string): HTMLElement | undefined {
  return (
    closestElement(range.startContainer, selector) ?? closestElement(range.commonAncestorContainer, selector)
  );
}

export function resolvePdfRangeLocation(range: Range, fallbackPage = 1): KnowledgeLocation {
  const pageElement = findClosestInRange(range, "[data-page-number]");
  const parsed = Number(pageElement?.dataset.pageNumber);
  const page = Number.isInteger(parsed) && parsed > 0 ? parsed : Math.max(1, fallbackPage);
  const rects = pageElement ? extractNormalizedPdfRects(range, pageElement) : [];
  return rects.length ? { kind: "pdf", page, rects } : { kind: "pdf", page };
}

export function resolveSlideRangeLocation(range: Range, fallbackSlide = 1): KnowledgeLocation {
  const slideElement = findClosestInRange(range, "[data-slide-number]");
  const parsed = Number(slideElement?.dataset.slideNumber);
  const slide = Number.isInteger(parsed) && parsed > 0 ? parsed : Math.max(1, fallbackSlide);
  return { kind: "slides", slide };
}

function computeTextOffsets(container: HTMLElement | undefined, selectedText: string) {
  const fullText = container?.textContent ?? "";
  const startIndex = selectedText ? fullText.indexOf(selectedText) : -1;
  return startIndex >= 0 ? { startOffset: startIndex, endOffset: startIndex + selectedText.length } : {};
}

export function resolveWebsiteRangeLocation(range: Range): KnowledgeLocation {
  const sectionElement = findClosestInRange(range, "[data-website-section]");
  const section = sectionElement?.dataset.websiteSection?.trim();
  const bodyElement =
    sectionElement?.querySelector<HTMLElement>("[data-website-section-body]") ?? sectionElement;
  return {
    kind: "website",
    ...(section ? { section } : {}),
    ...computeTextOffsets(bodyElement, range.toString().trim()),
  };
}

export function resolveMediaRangeLocation(range: Range, host: HTMLElement): KnowledgeLocation | undefined {
  const segmentElements = [...host.querySelectorAll<HTMLElement>("[data-segment-start-ms]")].filter(
    (element) => range.intersectsNode(element),
  );
  const first = segmentElements[0] ?? closestElement(range.startContainer, "[data-segment-start-ms]");
  const last =
    segmentElements.at(-1) ?? closestElement(range.endContainer, "[data-segment-start-ms]") ?? first;
  if (!first || !last) return undefined;
  const startMs = Number(first.dataset.segmentStartMs);
  const rawEndMs = Number(last.dataset.segmentEndMs);
  if (!Number.isFinite(startMs) || startMs < 0) return undefined;
  const endMs = Number.isFinite(rawEndMs) && rawEndMs > startMs ? rawEndMs : startMs + 1_000;
  return { kind: "media", startMs: Math.round(startMs), endMs: Math.round(endMs) };
}

function resolveCodeLineRange(range: Range): KnowledgeLocation | undefined {
  const startLineEl = closestElement(range.startContainer, "[data-code-line]");
  const endLineEl = closestElement(range.endContainer, "[data-code-line]") ?? startLineEl;
  const start = Number(startLineEl?.dataset.codeLine);
  const end = Number(endLineEl?.dataset.codeLine);
  if (!Number.isInteger(start) || start <= 0) return undefined;
  const resolvedEnd = Number.isInteger(end) && end > 0 ? end : start;
  return { kind: "code", lineStart: Math.min(start, resolvedEnd), lineEnd: Math.max(start, resolvedEnd) };
}

export function resolveChunkRangeLocation(
  range: Range,
  chunks: readonly KnowledgeChunk[],
  fallback: KnowledgeLocation,
): KnowledgeLocation {
  const codeLocation = resolveCodeLineRange(range);
  if (codeLocation) return codeLocation;
  const chunkElement = findClosestInRange(range, "[data-chunk-index]");
  const index = Number(chunkElement?.dataset.chunkIndex);
  const chunk = Number.isInteger(index) && index >= 0 ? chunks[index] : undefined;
  if (chunk) return chunk.locator.location;
  const selected = range.toString().trim();
  if (selected && chunks.length) {
    const matched = chunks.find((candidate) => candidate.text.includes(selected));
    if (matched) return matched.locator.location;
  }
  return fallback;
}

function nearestPrecedingHeading(container: HTMLElement | undefined, range: Range): string | undefined {
  if (!container) return undefined;
  let nearest: string | undefined;
  for (const heading of container.querySelectorAll<HTMLElement>("h1, h2, h3, h4")) {
    const relation = range.startContainer.compareDocumentPosition(heading);
    if (relation & Node.DOCUMENT_POSITION_PRECEDING || heading.contains(range.startContainer)) {
      nearest = heading.textContent?.trim() || nearest;
    }
  }
  return nearest;
}

export function resolveProseRangeLocation(range: Range, kind: "markdown" | "note"): KnowledgeLocation {
  const container = findClosestInRange(range, "[data-reader-prose]");
  const offsets = computeTextOffsets(container, range.toString().trim());
  if (kind === "markdown") {
    const heading = nearestPrecedingHeading(container, range);
    return { kind: "markdown", ...(heading ? { heading } : {}), ...offsets };
  }
  return { kind: "note", ...offsets };
}
