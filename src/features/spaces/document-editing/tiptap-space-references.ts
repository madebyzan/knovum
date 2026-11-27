import { search } from "fast-fuzzy";

import type { EditorContextReference } from "../../../shared/contracts/generated/EditorContextReference";
import { isSpaceSemanticNode } from "../knowledge/model/semantic-node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type {
  SpaceKnowledgeObjectPayload,
  SpaceNodeKind,
  SpaceNodePayload,
  SpaceWebsitePayload,
} from "../model/nodes/node-payload";

const MAX_EXCERPT_CHARS = 1200;

export interface SpaceAiReferenceCandidate extends EditorContextReference {
  readonly kind: SpaceNodeKind;
}

function compactText(value: string | undefined): string {
  const trimmed = value?.trim() ?? "";
  return trimmed.length <= MAX_EXCERPT_CHARS ? trimmed : trimmed.slice(0, MAX_EXCERPT_CHARS).trimEnd();
}

function websiteExcerpt(data: SpaceWebsitePayload): string {
  const sections = data.websiteSections?.map((section) => section.text).join("\n\n");
  return compactText(data.contentPreview ?? data.description ?? sections);
}

function objectExcerpt(data: SpaceKnowledgeObjectPayload): string {
  const details = Object.entries(data.properties)
    .filter(([, value]) => value !== null && String(value).trim() !== "")
    .map(([key, value]) => `${key}: ${String(value)}`);
  return compactText([data.summary, ...details].filter(Boolean).join("\n"));
}

const EXCERPT_BY_KIND: Partial<Record<SpaceNodeKind, (data: SpaceNodePayload) => string>> = {
  markdown: (data) => (data.kind === "markdown" ? compactText(data.text) : ""),
  note: (data) => (data.kind === "note" ? compactText(data.plainText) : ""),
  text: (data) => (data.kind === "text" ? compactText(data.plainText) : ""),
  website: (data) => (data.kind === "website" ? websiteExcerpt(data) : ""),
  file: (data) => (data.kind === "file" ? compactText(data.extractedText ?? data.previewText) : ""),
  code: (data) => (data.kind === "code" ? compactText(data.previewText) : ""),
  audio: (data) => (data.kind === "audio" ? compactText(data.previewText) : ""),
  video: (data) => (data.kind === "video" ? compactText(data.previewText) : ""),
  image: (data) =>
    data.kind === "image"
      ? compactText(
          data.visualKnowledge?.description ?? data.visualKnowledge?.detectedText ?? data.previewText,
        )
      : "",
  object: (data) => (data.kind === "object" ? objectExcerpt(data) : ""),
};

export function listSpaceAiReferenceCandidates(
  nodes: readonly SpaceCanvasNode[],
  excludeNodeId?: string,
): readonly SpaceAiReferenceCandidate[] {
  const candidates: SpaceAiReferenceCandidate[] = [];
  for (const node of nodes) {
    if (node.id === excludeNodeId || !isSpaceSemanticNode(node)) continue;
    const excerpt = EXCERPT_BY_KIND[node.data.kind]?.(node.data) ?? "";
    if (!excerpt) continue;
    const title = node.data.title.trim() || node.data.kind;
    candidates.push({ nodeId: node.id, title, excerpt, kind: node.data.kind });
  }
  return candidates;
}

export function filterSpaceAiReferences(
  candidates: readonly SpaceAiReferenceCandidate[],
  query: string,
): readonly SpaceAiReferenceCandidate[] {
  const trimmed = query.trim();
  if (!trimmed) return candidates;
  return search(trimmed, [...candidates], {
    keySelector: (candidate) => `${candidate.title} ${candidate.kind}`,
    threshold: 0.6,
  });
}
