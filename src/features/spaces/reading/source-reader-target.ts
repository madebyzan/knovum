import { classifyFilePresentation, fileExtension, usesDocumentPreview } from "../assets/file-presentation";
import { currentSourceRevision } from "../knowledge/model/provenance";
import type { KnowledgeLocation, KnowledgeLocator } from "../knowledge/model/schema/chunk";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { truncateUtf16 } from "../model/text";

export type SourceReaderKind =
  | "pdf"
  | "word"
  | "slides"
  | "website"
  | "media"
  | "image"
  | "document"
  | "code"
  | "markdown"
  | "note";

export type SourceReaderPane = "read" | "info";
export type SourceReaderHost = "canvas" | "document";

export interface SourceReadingTarget {
  nodeId: string;
  pane: SourceReaderPane;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
  nonce: number;
}

export interface SourceReaderRequest {
  nodeId: string;
  pane?: SourceReaderPane | undefined;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}

export interface SourceReadingHistoryEntry {
  nodeId: string;
  pane: SourceReaderPane;
}

function hasExtractedFileText(payload: Extract<SpaceNodePayload, { kind: "file" }>): boolean {
  return Boolean(payload.extractedText?.trim() || payload.previewText?.trim());
}

function classifyFileSource(
  payload: Extract<SpaceNodePayload, { kind: "file" }>,
): SourceReaderKind | undefined {
  if (payload.entries?.length) return undefined;
  const presentation = classifyFilePresentation(payload.fileName);
  if (presentation === "pdf") return "pdf";
  if (presentation === "presentation") return "slides";
  if (fileExtension(payload.fileName) === "docx" && payload.assetPath) return "word";
  return usesDocumentPreview(presentation) || presentation === "tabular-data" || hasExtractedFileText(payload)
    ? "document"
    : undefined;
}

function isMediaSourceReady(payload: Extract<SpaceNodePayload, { kind: "video" | "audio" }>): boolean {
  return !payload.processingState || Boolean(payload.sourceRevision);
}

const DIRECT_READER_KINDS: Partial<Record<SpaceNodePayload["kind"], SourceReaderKind>> = {
  website: "website",
  code: "code",
  markdown: "markdown",
  note: "note",
};

export function classifyReadableSource(payload: SpaceNodePayload): SourceReaderKind | undefined {
  if (payload.kind === "file") return classifyFileSource(payload);
  if (payload.kind === "video" || payload.kind === "audio") {
    return isMediaSourceReady(payload) ? "media" : undefined;
  }
  if (payload.kind === "image") return payload.assetPath ? "image" : undefined;
  return DIRECT_READER_KINDS[payload.kind];
}

export function isReadableSourceNode(node?: SpaceCanvasNode): boolean {
  return Boolean(node && classifyReadableSource(node.data) !== undefined);
}

export function buildSourceReaderLocator(
  payload: SpaceNodePayload,
  location: KnowledgeLocation,
  quoteText?: string,
): KnowledgeLocator {
  const exact = quoteText?.trim() ? truncateUtf16(quoteText.trim(), 64_000) : undefined;
  return {
    sourceRevision: currentSourceRevision(payload) ?? "node1:current",
    location,
    ...(exact ? { quote: { exact } } : {}),
  };
}

function defaultMediaLocation(payload: SpaceNodePayload): KnowledgeLocation {
  const durationSeconds =
    payload.kind === "video" || payload.kind === "audio" ? (payload.durationSeconds ?? 1) : 1;
  return { kind: "media", startMs: 0, endMs: Math.max(1_000, durationSeconds * 1_000) };
}

const STATIC_DEFAULT_LOCATIONS: Partial<Record<SourceReaderKind, KnowledgeLocation>> = {
  word: { kind: "word", section: 0, element: 0, headingPath: [] },
  slides: { kind: "slides", slide: 1 },
  website: { kind: "website" },
  code: { kind: "code", lineStart: 1, lineEnd: 1 },
  markdown: { kind: "markdown" },
  note: { kind: "note" },
};

export function defaultSourceLocation(payload: SpaceNodePayload, fallbackPage = 1): KnowledgeLocation {
  const kind = classifyReadableSource(payload);
  if (kind === "pdf") return { kind: "pdf", page: Math.max(1, fallbackPage) };
  if (kind === "media") return defaultMediaLocation(payload);
  return (kind ? STATIC_DEFAULT_LOCATIONS[kind] : undefined) ?? { kind: "node" };
}
