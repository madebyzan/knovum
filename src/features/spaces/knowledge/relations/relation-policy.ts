import { uniq } from "es-toolkit";

import {
  canonicalRelationEndpoints,
  type SpaceRelation,
  type SpaceRelationKind,
} from "../../model/relations/relation";

export interface CreateRelationInput {
  spaceId: string;
  sourceId: string;
  targetId: string;
  kind: SpaceRelationKind;
  rationale?: string;
}

export type RelationPatch = Partial<Pick<SpaceRelation, "kind" | "rationale" | "status" | "confidence">>;

export function createRelation(input: CreateRelationInput, relations: readonly SpaceRelation[]) {
  if (input.sourceId === input.targetId) return undefined;
  const [sourceId, targetId] = canonicalRelationEndpoints(input.sourceId, input.targetId, input.kind);
  const duplicate = findDuplicateRelation(relations, sourceId, targetId, input.kind);
  if (duplicate) return { kind: "duplicate" as const, relation: duplicate };
  const now = Date.now();
  const relation: SpaceRelation = {
    id: crypto.randomUUID(),
    spaceId: input.spaceId,
    sourceId,
    targetId,
    kind: input.kind,
    rationale: input.rationale ?? "",
    status: "active",
    actorKind: "human",
    createdAt: now,
    updatedAt: now,
  };
  return { kind: "created" as const, relation };
}

export function reviseRelation(
  current: SpaceRelation,
  patch: RelationPatch,
  relations: readonly SpaceRelation[],
) {
  const next: SpaceRelation = { ...current, ...patch, updatedAt: Date.now() };
  [next.sourceId, next.targetId] = canonicalRelationEndpoints(next.sourceId, next.targetId, next.kind);
  const duplicate =
    patch.kind && patch.kind !== current.kind
      ? findDuplicateRelation(
          relations.filter(({ id }) => id !== current.id),
          next.sourceId,
          next.targetId,
          next.kind,
        )
      : undefined;
  return duplicate
    ? { kind: "duplicate" as const, relation: duplicate }
    : { kind: "updated" as const, relation: next };
}

export function selectRelationsForSpace(
  relations: readonly SpaceRelation[],
  ids: readonly string[],
  spaceId: string | undefined,
) {
  const uniqueIds = uniq(ids);
  const selected = uniqueIds.flatMap((id) => {
    const relation = relations.find((candidate) => candidate.id === id);
    return relation ? [relation] : [];
  });
  if (!selected.length) return undefined;
  if (selected.length !== uniqueIds.length) throw new Error("Selected Space Relation no longer exists");
  if (!spaceId || selected.some((relation) => relation.spaceId !== spaceId))
    throw new Error("Selected Space Relations must belong to the active Space");
  return { ids: uniqueIds, relations: selected };
}

function findDuplicateRelation(
  relations: readonly SpaceRelation[],
  sourceId: string,
  targetId: string,
  kind: SpaceRelationKind,
) {
  const [left, right] = canonicalRelationEndpoints(sourceId, targetId, kind);
  return relations.find((relation) => {
    if (relation.kind !== kind || relation.status === "invalidated") return false;
    const endpoints = canonicalRelationEndpoints(relation.sourceId, relation.targetId, kind);
    return endpoints[0] === left && endpoints[1] === right;
  });
}
