import { invoke } from "@tauri-apps/api/core";
import { pick } from "es-toolkit";

import type { SpaceRelation } from "../../model/relations/relation";

export function listRelations(spaceId: string): Promise<SpaceRelation[]> {
  return invoke("plugin:spaces|list_relations", { spaceId });
}

function relationInput(relation: SpaceRelation) {
  return {
    ...pick(relation, ["id", "spaceId", "sourceId", "targetId", "kind", "rationale", "status"]),
    confidence: relation.confidence ?? null,
  };
}

export function createRelation(relation: SpaceRelation): Promise<void> {
  return invoke("plugin:spaces|create_relation", { relation: relationInput(relation) });
}

export function removeRelation(relationId: string): Promise<void> {
  return invoke("plugin:spaces|revoke_relation", { relationId });
}

export function restoreRelation(relationId: string): Promise<void> {
  return invoke("plugin:spaces|restore_relation", { relationId });
}

export function updateRelation(relation: SpaceRelation): Promise<void> {
  return invoke("plugin:spaces|update_relation", { relation: relationInput(relation) });
}
