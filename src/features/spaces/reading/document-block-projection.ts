import type { KnowledgeLocation } from "../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../knowledge/model/schema/persisted-chunk";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { defaultSourceLocation } from "./source-reader-target";

export interface DocumentRenderBlock {
  id: string;
  chunkIndex: number;
  kind: "heading" | "paragraph" | "table" | "code" | "slide";
  sectionLabel?: string | undefined;
  slideNumber?: number | undefined;
  text: string;
  tableRows?: readonly (readonly string[])[] | undefined;
  codeLines?: readonly { number: number; text: string }[] | undefined;
  location: KnowledgeLocation;
}

const TABLE_HEADER_PREFIX = /^Table header:\n[^\n]*\n\n/u;

function parseTableRows(text: string): readonly (readonly string[])[] {
  const cleaned = text.replace(TABLE_HEADER_PREFIX, "").trim();
  return cleaned
    .split(/\r?\n/u)
    .map((line) => line.split("\t").map((cell) => cell.trim()))
    .filter((row) => row.some(Boolean));
}

function parseCodeLines(text: string, startLine: number) {
  return text.split(/\r?\n/u).map((line, offset) => ({
    number: startLine + offset,
    text: line,
  }));
}

function blockSectionLabel(location: KnowledgeLocation): string | undefined {
  if (location.kind === "word") {
    return location.headingPath.length ? location.headingPath.join(" / ") : undefined;
  }
  return location.kind === "markdown" ? location.heading : undefined;
}

function isTableChunk(chunk: KnowledgeChunk): boolean {
  return chunk.kind === "table" || (chunk.text.includes("\t") && chunk.text.includes("\n"));
}

function chunkToRenderBlock(chunk: KnowledgeChunk, chunkIndex: number): DocumentRenderBlock {
  const loc = chunk.locator.location;
  if (loc.kind === "code" || chunk.kind === "code") {
    const startLine = loc.kind === "code" ? loc.lineStart : 1;
    return {
      id: chunk.id,
      chunkIndex,
      kind: "code",
      text: chunk.text,
      codeLines: parseCodeLines(chunk.text, startLine),
      location: loc,
    };
  }
  if (isTableChunk(chunk)) {
    return {
      id: chunk.id,
      chunkIndex,
      kind: "table",
      sectionLabel: blockSectionLabel(loc),
      slideNumber: loc.kind === "slides" ? loc.slide : undefined,
      text: chunk.text,
      tableRows: parseTableRows(chunk.text),
      location: loc,
    };
  }
  if (loc.kind === "slides") {
    return {
      id: chunk.id,
      chunkIndex,
      kind: "slide",
      slideNumber: loc.slide,
      text: chunk.text,
      location: loc,
    };
  }
  return {
    id: chunk.id,
    chunkIndex,
    kind: chunk.kind === "heading" ? "heading" : "paragraph",
    sectionLabel: blockSectionLabel(loc),
    text: chunk.text,
    location: loc,
  };
}

function fallbackPayloadText(payload: SpaceNodePayload): string | undefined {
  if (payload.kind === "file") return payload.extractedText?.trim() || payload.previewText?.trim();
  return payload.kind === "code" ? payload.previewText?.trim() : undefined;
}

export function projectDocumentBlocks(
  chunks: readonly KnowledgeChunk[],
  payload: SpaceNodePayload,
): readonly DocumentRenderBlock[] {
  const contentEntries = chunks
    .map((chunk, index) => ({ chunk, index }))
    .filter(({ chunk }) => chunk.kind !== "node");
  if (contentEntries.length) {
    return contentEntries.map(({ chunk, index }) => chunkToRenderBlock(chunk, index));
  }
  const fallbackText = fallbackPayloadText(payload);
  if (!fallbackText) return [];
  if (payload.kind === "code") {
    const lines = parseCodeLines(fallbackText, 1);
    return [
      {
        id: "fallback-code",
        chunkIndex: 0,
        kind: "code",
        text: fallbackText,
        codeLines: lines,
        location: { kind: "code", lineStart: 1, lineEnd: Math.max(1, lines.length) },
      },
    ];
  }
  return [
    {
      id: "fallback-doc",
      chunkIndex: 0,
      kind: "paragraph",
      text: fallbackText,
      location: defaultSourceLocation(payload),
    },
  ];
}
