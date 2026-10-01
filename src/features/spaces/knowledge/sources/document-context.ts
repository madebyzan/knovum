import type { KnowledgeLocation } from "../model/schema/chunk";
import type { KnowledgeChunk } from "../model/schema/persisted-chunk";
import { listCurrentKnowledgeChunks } from "../persistence/chunk-operations";

export async function loadDocumentContext(
  request:
    | {
        spaceId: string;
        nodeId: string;
        location: KnowledgeLocation;
      }
    | undefined,
) {
  if (!request) return [];
  const chunks = await listCurrentKnowledgeChunks(request.spaceId, [request.nodeId]);
  return documentContext(chunks, request.location);
}

function documentContext(chunks: KnowledgeChunk[], target: KnowledgeLocation) {
  if (target.kind === "pdf")
    return chunks.filter(
      ({ locator }) => locator.location.kind === "pdf" && locator.location.page === target.page,
    );
  if (target.kind === "word")
    return chunks.filter(
      ({ locator }) =>
        locator.location.kind === "word" &&
        locator.location.section === target.section &&
        Math.abs(locator.location.element - target.element) <= 2,
    );
  if (target.kind === "slides")
    return chunks.filter(
      ({ locator }) => locator.location.kind === "slides" && locator.location.slide === target.slide,
    );
  if (target.kind === "sheet")
    return chunks.filter(({ locator }) => {
      const location = locator.location;
      return (
        location.kind === "sheet" &&
        location.sheet === target.sheet &&
        (target.element === undefined ||
          location.element === undefined ||
          Math.abs(location.element - target.element) <= 2)
      );
    });
  return [];
}
