import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { KnowledgeLocator } from "./schema";

export function isLocatorCurrent(locator: KnowledgeLocator, data?: SpaceNodePayload) {
  if (!data) return false;
  // Semantic revisions have an explicit namespace; document-derived citations still point to
  // pages/elements, but are invalidated by the semantic aggregate rather than the original bytes.
  const expected = locator.sourceRevision.startsWith("node1:") ? data.knowledgeRevision : data.sourceRevision;
  return expected === locator.sourceRevision;
}
