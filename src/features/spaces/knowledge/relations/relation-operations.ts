import { pick } from "es-toolkit";

import { invokeCommand as invoke } from "../../../../foundation/ipc/invoke-command";
import type { SpaceRelation } from "../../model/relations/relation";

function relationInput(relation: SpaceRelation) {
  return {
    ...pick(relation, ["id", "spaceId", "sourceId", "targetId", "kind", "rationale", "status"]),
    confidence: relation.confidence ?? null,
  };
}

export function createRelation(relation: SpaceRelation): Promise<void> {
  return invoke("plugin:spaces|create_relation", { relation: relationInput(relation) });
}

export function revokeRelation(relationId: string): Promise<void> {
  return invoke("plugin:spaces|revoke_relation", { relationId });
}

export function revokeRelations(relationIds: readonly string[]): Promise<void> {
  return invoke("plugin:spaces|revoke_relations", { relationIds });
}

export function restoreRelation(relationId: string): Promise<void> {
  return invoke("plugin:spaces|restore_relation", { relationId });
}

export function restoreRelations(relationIds: readonly string[]): Promise<void> {
  return invoke("plugin:spaces|restore_relations", { relationIds });
}

export function updateRelation(relation: SpaceRelation): Promise<void> {
  return invoke("plugin:spaces|update_relation", { relation: relationInput(relation) });
}
