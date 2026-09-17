import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { knowledgeRevision } from "./revision";

/** UX-side precomputation only. Rust recomputes and validates the durable revision. */
export function prepareSpaceKnowledgePayload(payload: SpaceNodePayload): SpaceNodePayload {
  const revision = knowledgeRevision(payload);
  if (!revision) return payload;
  return payload.knowledgeRevision === revision ? payload : { ...payload, knowledgeRevision: revision };
}
