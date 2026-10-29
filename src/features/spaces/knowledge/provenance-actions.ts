import { queryClient } from "../../../foundation/query/query-client";
import type { KnowledgePeekTarget } from "./model/knowledge-peek-target";
import type { ProvenanceSource, SpaceProvenance, SpaceProvenanceTargetKind } from "./model/provenance";
import {
  createProvenance,
  revokeProvenance,
  type CreateProvenanceInput,
} from "./persistence/knowledge-operations";
import { SPACE_KNOWLEDGE_QUERY_KEYS } from "./queries/query-options";

interface ProvenanceTarget {
  spaceId: string;
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
}

export async function attachProvenance(target: ProvenanceTarget, source: ProvenanceSource) {
  const provenance: CreateProvenanceInput = {
    id: crypto.randomUUID(),
    ...target,
    sourceNodeId: source.nodeId,
    locator: source.locator,
    sourceRevision: source.locator.sourceRevision,
    quote: source.quote,
  };
  await createProvenance(provenance);
  await refreshProvenance(provenance);
}

export function attachProvenanceFromPeek(target: ProvenanceTarget, peek: KnowledgePeekTarget) {
  return attachProvenance(target, {
    nodeId: peek.nodeId,
    locator: peek.locator,
    quote: peek.locator.quote?.exact ?? peek.text ?? "",
  });
}

/**
 * Takes `record` again at the source's `revision`: the person has seen the source change and keeps
 * what the target says. The cited words stay as the record of what was taken; where they were in
 * the old revision says nothing about the new one, so the record now points at the whole source.
 */
export function reviewProvenance(record: SpaceProvenance, revision: string) {
  const { spaceId, targetKind, targetId, sourceNodeId, quote } = record;
  return attachProvenance(
    { spaceId, targetKind, targetId },
    { nodeId: sourceNodeId, locator: { sourceRevision: revision, location: { kind: "node" } }, quote },
  );
}

export async function detachProvenance(provenanceId: string, target: ProvenanceTarget) {
  await revokeProvenance(provenanceId);
  await refreshProvenance(target);
}

// What is recorded about the target, and whether its sources have changed since.
function refreshProvenance(target: ProvenanceTarget) {
  return Promise.all([
    queryClient.invalidateQueries({
      queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.provenance(target.spaceId, target.targetKind, target.targetId),
    }),
    queryClient.invalidateQueries({ queryKey: SPACE_KNOWLEDGE_QUERY_KEYS.health(target.spaceId) }),
  ]);
}
