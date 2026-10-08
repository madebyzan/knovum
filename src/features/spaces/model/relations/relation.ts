import { relationKinds, relationStatuses } from "../../contracts/generated/contract-values";
import type { KnowledgeActorKind } from "../../contracts/generated/KnowledgeActorKind";
import type { RelationKind } from "../../contracts/generated/RelationKind";
import type { RelationStatus } from "../../contracts/generated/RelationStatus";
import { compareUtf8Binary } from "../text";

export const SPACE_RELATION_KINDS = relationKinds;
export type SpaceRelationKind = RelationKind;

export function isSymmetricRelationKind(kind: unknown): kind is "related" | "contradicts" {
  return kind === "related" || kind === "contradicts";
}

/** Kept as the Relation-domain name for callers; ordering is shared by all UTF-8 durable contracts. */
export const compareRelationIdsBinary = compareUtf8Binary;

export function canonicalRelationEndpoints(
  sourceId: string,
  targetId: string,
  kind: SpaceRelationKind,
): [string, string] {
  return isSymmetricRelationKind(kind) && compareRelationIdsBinary(sourceId, targetId) > 0
    ? [targetId, sourceId]
    : [sourceId, targetId];
}

export const SPACE_RELATION_STATUSES = relationStatuses;
export type SpaceRelationStatus = RelationStatus;

export interface SpaceRelation {
  id: string;
  spaceId: string;
  sourceId: string;
  targetId: string;
  kind: SpaceRelationKind;
  rationale: string;
  status: SpaceRelationStatus;
  statusBeforeInvalidation?: "active" | "contested" | null;
  confidence?: number;
  actorKind: KnowledgeActorKind;
  actorId?: string;
  createdAt: number;
  updatedAt: number;
}
