import { indexKnowledgeRetrieval } from "../persistence/retrieval-index";

/**
 * Settles the retrieval index of a Space on demand and reports how many chunks had to be encoded.
 *
 * Indexing picks the smallest build that can make the Space searchable: the gaps of the active
 * generation are filled, and every chunk is re-encoded only when no generation can be reused — a
 * changed embedding spec, no active generation, or a lost vector table.
 */
export function repairKnowledgeRetrievalIndex(spaceId: string): Promise<{ indexed: number }> {
  return indexKnowledgeRetrieval(spaceId);
}
