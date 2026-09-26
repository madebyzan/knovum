import { queryClient } from "../../../foundation/query/query-client";
import type { SpaceProvenanceTargetKind } from "./model/provenance";
import {
  createProvenance,
  removeProvenance,
  type CreateProvenanceInput,
} from "./persistence/knowledge-operations";
import { spaceKnowledgeQueryKeys } from "./queries/query-options";

interface ProvenanceTarget {
  spaceId: string;
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
}

export async function attachProvenance(provenance: CreateProvenanceInput) {
  await createProvenance(provenance);
  await refreshProvenance(provenance);
}

export async function detachProvenance(id: string, target: ProvenanceTarget) {
  await removeProvenance(id);
  await refreshProvenance(target);
}

function refreshProvenance(target: ProvenanceTarget) {
  return queryClient.invalidateQueries({
    queryKey: spaceKnowledgeQueryKeys.provenance(target.spaceId, target.targetKind, target.targetId),
  });
}
