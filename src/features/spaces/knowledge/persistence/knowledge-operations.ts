import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { KnowledgeChangeSet } from "../../contracts/generated/KnowledgeChangeSet";
import type { SpaceKnowledgeHealthIssue } from "../model/health";
import type { SpaceKnowledgeObjectType } from "../model/object-types";
import type { SpaceKnowledgeEvent, SpaceProvenance } from "../model/provenance";
import type { KnowledgeLocator } from "../model/schema";

export interface ObjectTypeWriteInput {
  id: string;
  spaceId: string;
  name: string;
  icon: string;
  description: string;
  fields: SpaceKnowledgeObjectType["fields"];
}

export interface CreateProvenanceInput {
  id: string;
  spaceId: string;
  targetKind: "node" | "relation";
  targetId: string;
  sourceNodeId: string;
  locator: KnowledgeLocator;
  sourceRevision: string;
  quote: string;
}

export interface ListKnowledgeEventsQuery {
  spaceId: string;
  entityKind?: SpaceKnowledgeEvent["entityKind"];
  entityId?: string;
  limit?: number;
}

export function listObjectTypes(spaceId: string): Promise<SpaceKnowledgeObjectType[]> {
  return invokeCommand("plugin:spaces|list_object_types", { spaceId });
}

export function createObjectType(input: ObjectTypeWriteInput): Promise<void> {
  return invokeCommand("plugin:spaces|create_object_type", { input });
}

export function updateObjectType(input: ObjectTypeWriteInput): Promise<void> {
  return invokeCommand("plugin:spaces|update_object_type", { input });
}

export function deleteObjectType(objectTypeId: string, spaceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|delete_object_type", { objectTypeId, spaceId });
}

export function listProvenance(
  spaceId: string,
  targetKind: "node" | "relation",
  targetId: string,
): Promise<SpaceProvenance[]> {
  return invokeCommand("plugin:spaces|list_provenance", { spaceId, targetKind, targetId });
}

export function createProvenance(input: CreateProvenanceInput): Promise<void> {
  return invokeCommand("plugin:spaces|create_provenance", { input });
}

export function revokeProvenance(provenanceId: string): Promise<void> {
  return invokeCommand("plugin:spaces|revoke_provenance", { provenanceId });
}

export function listKnowledgeEvents(query: ListKnowledgeEventsQuery): Promise<SpaceKnowledgeEvent[]> {
  const { spaceId, entityKind, entityId, limit = 100 } = query;
  return invokeCommand("plugin:spaces|list_knowledge_events", {
    request: {
      spaceId,
      entityKind: entityKind ?? null,
      entityId: entityId ?? null,
      limit,
    },
  });
}

export function commitKnowledgeChanges(
  changes: KnowledgeChangeSet,
): Promise<{ resolvedIds: Record<string, string> }> {
  return invokeCommand("plugin:spaces|commit_knowledge_changes", { changes });
}

export function listKnowledgeHealthIssues(spaceId: string): Promise<SpaceKnowledgeHealthIssue[]> {
  return invokeCommand("plugin:spaces|list_knowledge_health_issues", { spaceId });
}
