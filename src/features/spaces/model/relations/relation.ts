import { compareUtf8Binary } from "../text";

export const SPACE_RELATION_KINDS = [
  "reference",
  "related",
  "supports",
  "contradicts",
  "depends-on",
  "supersedes",
  "answers",
  "uses",
  "derives-from",
  "caused-by",
] as const;
export type SpaceRelationKind = (typeof SPACE_RELATION_KINDS)[number];

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

export const SPACE_RELATION_STATUSES = ["active", "contested", "invalidated"] as const;
export type SpaceRelationStatus = (typeof SPACE_RELATION_STATUSES)[number];
type SpaceRelationActorKind = "human" | "assistant" | "agent" | "system";

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
  actorKind: SpaceRelationActorKind;
  actorId?: string;
  createdAt: number;
  updatedAt: number;
}
