import { decodeAppError } from "../../../../foundation/errors/app-error";

/**
 * The outcomes `index_knowledge` reports that are not failures.
 *
 * A pulled token and a Space that is already being indexed are both answers to the question the
 * caller asked — one says the work stopped where it was committed, the other says the work is
 * already in flight — so every caller recognises them here rather than inventing its own reading
 * of an error message.
 */
export function isKnowledgeIndexingCancelled(error: unknown): boolean {
  return decodeAppError(error)?.code === "cancelled";
}

export function isKnowledgeIndexingBusy(error: unknown): boolean {
  return decodeAppError(error)?.code === "knowledge_indexing_busy";
}
