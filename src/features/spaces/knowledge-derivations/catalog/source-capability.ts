import type { SpaceNodePayload } from "../../model/nodes/node-payload";

const KNOWLEDGE_PREDICATES: ReadonlyArray<(data: SpaceNodePayload) => boolean> = [
  (data) => data.kind === "note" && Boolean(data.plainText.trim()),
  (data) => data.kind === "markdown" && Boolean(data.text.trim()),
  (data) => data.kind === "website" && Boolean(data.contentPreview?.trim() || data.websiteSections?.length),
  (data) => data.kind === "image" && Boolean(data.visualKnowledge),
  (data) => data.kind === "video" && Boolean(data.mediaKnowledge || data.visualKnowledge),
  (data) => data.kind === "audio" && Boolean(data.mediaKnowledge),
  (data) => data.kind === "file" && Boolean(data.extractedText?.trim()),
  (data) => data.kind === "code" && Boolean(data.previewText?.trim()),
  (data) => data.kind === "object",
];

export function hasDerivableKnowledge(data: SpaceNodePayload): boolean {
  return KNOWLEDGE_PREDICATES.some((predicate) => predicate(data));
}
