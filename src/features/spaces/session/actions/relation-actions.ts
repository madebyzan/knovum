import type { Ref } from "vue";

import { recordSpaceHistory, type SpaceHistoryState } from "../../history/stack";
import {
  createRelation as persistRelation,
  removeRelation,
  removeRelations,
  restoreRelation,
  restoreRelations,
  updateRelation,
} from "../../knowledge/relations/relation-operations";
import { canonicalRelationEndpoints } from "../../model/relations/relation";
import type { SpaceRelation, SpaceRelationKind } from "../../model/relations/relation";

export interface RelationContext {
  activeSpaceId: Ref<string | undefined>;
  relations: Ref<SpaceRelation[]>;
  history: SpaceHistoryState;
}

interface CreateRelationInput {
  sourceId: string;
  targetId: string;
  kind: SpaceRelationKind;
  rationale?: string;
  spaceId?: string;
}

export async function createSpaceRelation(context: RelationContext, input: CreateRelationInput) {
  const spaceId = input.spaceId ?? context.activeSpaceId.value;
  if (!spaceId || input.sourceId === input.targetId) return;
  const [sourceId, targetId] = canonicalRelationEndpoints(input.sourceId, input.targetId, input.kind);
  const duplicate =
    context.activeSpaceId.value === spaceId
      ? findDuplicate(context.relations.value, sourceId, targetId, input.kind)
      : undefined;
  if (duplicate) return duplicate;
  const now = Date.now();
  const relation: SpaceRelation = {
    id: crypto.randomUUID(),
    spaceId,
    sourceId,
    targetId,
    kind: input.kind,
    rationale: input.rationale ?? "",
    status: "active",
    actorKind: "human",
    createdAt: now,
    updatedAt: now,
  };
  await persistRelation(relation);
  if (context.activeSpaceId.value === spaceId) {
    addRelation(context, relation);
    recordSpaceHistory(context.history, historyForInsert(context, relation));
  }
  return relation;
}

export async function changeSpaceRelationKind(context: RelationContext, id: string, kind: SpaceRelationKind) {
  const current = context.relations.value.find((relation) => relation.id === id);
  if (!current || current.kind === kind) return;
  const duplicate = findDuplicate(
    context.relations.value.filter((relation) => relation.id !== id),
    current.sourceId,
    current.targetId,
    kind,
  );
  if (duplicate) {
    await removeSpaceRelation(context, id);
    return duplicate;
  }
  return updateSpaceRelation(context, id, { kind });
}

export async function updateSpaceRelation(
  context: RelationContext,
  id: string,
  patch: Partial<Pick<SpaceRelation, "kind" | "rationale" | "status" | "confidence">>,
) {
  const current = context.relations.value.find((relation) => relation.id === id);
  if (!current) return;
  const next: SpaceRelation = { ...current, ...patch, updatedAt: Date.now() };
  [next.sourceId, next.targetId] = canonicalRelationEndpoints(next.sourceId, next.targetId, next.kind);
  await updateRelation(next);
  if (context.activeSpaceId.value === current.spaceId) {
    replaceRelation(context, next);
    recordSpaceHistory(context.history, {
      undo: () => persistState(context, current),
      redo: () => persistState(context, next),
    });
  }
  return next;
}

export async function removeSpaceRelation(context: RelationContext, id: string) {
  await removeSpaceRelations(context, [id]);
}

export async function removeSpaceRelations(context: RelationContext, ids: readonly string[]) {
  const uniqueIds = [...new Set(ids)];
  const selected = uniqueIds.flatMap((id) => {
    const relation = context.relations.value.find((candidate) => candidate.id === id);
    return relation ? [relation] : [];
  });
  if (!selected.length) return;
  if (selected.length !== uniqueIds.length) throw new Error("Selected Space Relation no longer exists");
  const activeSpaceId = context.activeSpaceId.value;
  if (!activeSpaceId || selected.some(({ spaceId }) => spaceId !== activeSpaceId))
    throw new Error("Selected Space Relations must belong to the active Space");
  await removeRelations(uniqueIds);
  removeRelationsFromState(context, uniqueIds);
  recordSpaceHistory(context.history, {
    undo: async () => {
      await restoreRelations(uniqueIds);
      for (const relation of selected) addRelation(context, relation);
    },
    redo: async () => {
      await removeRelations(uniqueIds);
      removeRelationsFromState(context, uniqueIds);
    },
  });
}

function findDuplicate(
  relations: readonly SpaceRelation[],
  source: string,
  target: string,
  kind: SpaceRelationKind,
) {
  const [left, right] = canonicalRelationEndpoints(source, target, kind);
  return relations.find((relation) => {
    if (relation.kind !== kind || relation.status === "invalidated") return false;
    const [existingLeft, existingRight] = canonicalRelationEndpoints(
      relation.sourceId,
      relation.targetId,
      kind,
    );
    return existingLeft === left && existingRight === right;
  });
}

function historyForInsert(context: RelationContext, relation: SpaceRelation) {
  return {
    undo: async () => {
      await removeRelation(relation.id);
      removeRelationFromState(context, relation);
    },
    redo: async () => {
      await restoreRelation(relation.id);
      addRelation(context, relation);
    },
  };
}

async function persistState(context: RelationContext, relation: SpaceRelation) {
  await updateRelation(relation);
  if (context.activeSpaceId.value === relation.spaceId) replaceRelation(context, relation);
}

function addRelation(context: RelationContext, relation: SpaceRelation) {
  if (
    context.activeSpaceId.value === relation.spaceId &&
    !context.relations.value.some((candidate) => candidate.id === relation.id)
  ) {
    context.relations.value = [...context.relations.value, relation];
  }
}

function removeRelationFromState(context: RelationContext, relation: SpaceRelation) {
  if (context.activeSpaceId.value !== relation.spaceId) return;
  context.relations.value = context.relations.value.filter((candidate) => candidate.id !== relation.id);
}

function removeRelationsFromState(context: RelationContext, ids: readonly string[]) {
  const removed = new Set(ids);
  context.relations.value = context.relations.value.filter(({ id }) => !removed.has(id));
}

function replaceRelation(context: RelationContext, next: SpaceRelation) {
  if (context.activeSpaceId.value !== next.spaceId) return;
  context.relations.value = context.relations.value.map((relation) =>
    relation.id === next.id ? next : relation,
  );
}
