import type { KnowledgeActorKind } from "../../contracts/generated/KnowledgeActorKind";
import type { RelationKind } from "../../contracts/generated/RelationKind";
import type { RelationStatus } from "../../contracts/generated/RelationStatus";
import { compareUtf8Binary } from "../text";

export function isSymmetricRelationKind(kind: unknown): kind is "related" | "contradicts" {
  return kind === "related" || kind === "contradicts";
}

export function canonicalRelationEndpoints(
  sourceId: string,
  targetId: string,
  kind: RelationKind,
): [string, string] {
  return isSymmetricRelationKind(kind) && compareUtf8Binary(sourceId, targetId) > 0
    ? [targetId, sourceId]
    : [sourceId, targetId];
}

export interface SpaceRelation {
  id: string;
  spaceId: string;
  sourceId: string;
  targetId: string;
  kind: RelationKind;
  rationale: string;
  status: RelationStatus;
  statusBeforeInvalidation?: "active" | "contested" | null;
  confidence?: number;
  actorKind: KnowledgeActorKind;
  actorId?: string;
  createdAt: number;
  updatedAt: number;
}
