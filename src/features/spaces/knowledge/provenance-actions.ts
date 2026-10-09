import { queryClient } from "../../../foundation/query/query-client";
import type { KnowledgePeekTarget } from "./model/knowledge-peek-target";
import type { SpaceProvenanceTargetKind } from "./model/provenance";
import {
  createProvenance,
  revokeProvenance,
  type CreateProvenanceInput,
} from "./persistence/knowledge-operations";
import { spaceKnowledgeQueryKeys } from "./queries/query-options";

interface ProvenanceTarget {
  spaceId: string;
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
}

export async function attachProvenanceFromPeek(target: ProvenanceTarget, peek: KnowledgePeekTarget) {
  const provenance: CreateProvenanceInput = {
    id: crypto.randomUUID(),
    ...target,
    sourceNodeId: peek.nodeId,
    locator: peek.locator,
    sourceRevision: peek.locator.sourceRevision,
    quote: peek.locator.quote?.exact ?? peek.text ?? "",
  };
  await createProvenance(provenance);
  await refreshProvenance(provenance);
}

export async function detachProvenance(provenanceId: string, target: ProvenanceTarget) {
  await revokeProvenance(provenanceId);
  await refreshProvenance(target);
}

function refreshProvenance(target: ProvenanceTarget) {
  return queryClient.invalidateQueries({
    queryKey: spaceKnowledgeQueryKeys.provenance(target.spaceId, target.targetKind, target.targetId),
  });
}
