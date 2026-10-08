import type { KnowledgeActorKind } from "../../contracts/generated/KnowledgeActorKind";
import type { KnowledgeLocator } from "./schema/chunk";

type SpaceProvenanceStatus = "active" | "superseded" | "invalidated";
export type SpaceProvenanceTargetKind = "node" | "relation";

export interface SpaceProvenance {
  id: string;
  spaceId: string;
  targetKind: SpaceProvenanceTargetKind;
  targetId: string;
  sourceNodeId: string;
  locator: KnowledgeLocator;
  sourceRevision: string;
  quote: string;
  status: SpaceProvenanceStatus;
  actorKind: KnowledgeActorKind;
  actorId?: string;
  createdAt: number;
  updatedAt: number;
}

type SpaceKnowledgeEventKind =
  | "created"
  | "updated"
  | "drafted"
  | "accepted"
  | "contested"
  | "superseded"
  | "invalidated"
  | "opened"
  | "resolved"
  | "restored"
  | "linked"
  | "unlinked"
  | "applied"
  | "rejected"
  | "moved";

export interface SpaceKnowledgeEvent {
  id: string;
  spaceId: string;
  entityKind: "node" | "relation" | "provenance" | "type";
  entityId: string;
  eventKind: SpaceKnowledgeEventKind;
  actorKind: KnowledgeActorKind;
  actorId?: string;
  payload: unknown;
  createdAt: number;
}
