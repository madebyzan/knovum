import type { Ref } from "vue";

import { recordSpaceHistory, type SpaceHistoryState } from "../../history/stack";
import {
  createRelation as persistRelation,
  revokeRelation,
  revokeRelations,
  restoreRelation,
  restoreRelations,
  updateRelation,
} from "../../knowledge/relations/relation-operations";
import {
  createRelation,
  reviseRelation,
  selectRelationsForSpace,
  type CreateRelationInput as OwnedCreateRelationInput,
  type RelationPatch,
} from "../../knowledge/relations/relation-policy";
import type { SpaceRelation, SpaceRelationKind } from "../../model/relations/relation";

export interface RelationContext {
  activeSpaceId: Ref<string | undefined>;
  relations: Ref<SpaceRelation[]>;
  history: SpaceHistoryState;
}

interface CreateRelationInput extends Omit<OwnedCreateRelationInput, "spaceId"> {
  spaceId?: string;
}

export async function createSpaceRelation(context: RelationContext, input: CreateRelationInput) {
  const spaceId = input.spaceId ?? context.activeSpaceId.value;
  if (!spaceId) return;
  const decision = createRelation(
    {
      spaceId,
      sourceId: input.sourceId,
      targetId: input.targetId,
      kind: input.kind,
      ...(input.rationale === undefined ? {} : { rationale: input.rationale }),
    },
    context.activeSpaceId.value === spaceId ? context.relations.value : [],
  );
  if (!decision || decision.kind === "duplicate") return decision?.relation;
  const relation = decision.relation;
  await persistRelation(relation);
  if (context.activeSpaceId.value === spaceId) {
    addRelation(context, relation);
    recordSpaceHistory(context.history, historyForInsert(context, relation));
  }
  return relation;
}

export async function changeSpaceRelationKind(
  context: RelationContext,
  relationId: string,
  kind: SpaceRelationKind,
) {
  const current = context.relations.value.find((relation) => relation.id === relationId);
  if (!current || current.kind === kind) return;
  return updateSpaceRelation(context, relationId, { kind });
}

export async function updateSpaceRelation(
  context: RelationContext,
  relationId: string,
  patch: RelationPatch,
) {
  const current = context.relations.value.find((relation) => relation.id === relationId);
  if (!current) return;
  const decision = reviseRelation(current, patch, context.relations.value);
  if (decision.kind === "duplicate") {
    await removeSpaceRelation(context, relationId);
    return decision.relation;
  }
  const next = decision.relation;
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

export async function removeSpaceRelation(context: RelationContext, relationId: string) {
  await removeSpaceRelations(context, [relationId]);
}

export async function removeSpaceRelations(context: RelationContext, relationIds: readonly string[]) {
  const selection = selectRelationsForSpace(
    context.relations.value,
    relationIds,
    context.activeSpaceId.value,
  );
  if (!selection) return;
  await revokeRelations(selection.ids);
  removeRelationsFromState(context, selection.ids);
  recordSpaceHistory(context.history, {
    undo: async () => {
      await restoreRelations(selection.ids);
      for (const relation of selection.relations) addRelation(context, relation);
    },
    redo: async () => {
      await revokeRelations(selection.ids);
      removeRelationsFromState(context, selection.ids);
    },
  });
}

function historyForInsert(context: RelationContext, relation: SpaceRelation) {
  return {
    undo: async () => {
      await revokeRelation(relation.id);
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

function removeRelationsFromState(context: RelationContext, relationIds: readonly string[]) {
  const removed = new Set(relationIds);
  context.relations.value = context.relations.value.filter(({ id }) => !removed.has(id));
}

function replaceRelation(context: RelationContext, next: SpaceRelation) {
  if (context.activeSpaceId.value !== next.spaceId) return;
  context.relations.value = context.relations.value.map((relation) =>
    relation.id === next.id ? next : relation,
  );
}
