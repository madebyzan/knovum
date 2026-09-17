import type { SpaceJsonValue } from "../../model/json-value";
import type { SpaceRelationKind } from "../../model/relations/relation";
import type { SpaceKnowledgeObjectStatus } from "./object-types";
import type { KnowledgeLocator } from "./schema";

export type KnowledgeChange =
  | {
      op: "create-object";
      tempId: string;
      objectTypeId: string;
      title: string;
      summary?: string;
      status: SpaceKnowledgeObjectStatus;
      properties: Record<string, SpaceJsonValue>;
      position?: { x: number; y: number };
    }
  | {
      op: "update-object";
      nodeId: string;
      title?: string;
      summary?: string;
      status?: SpaceKnowledgeObjectStatus;
      properties?: Record<string, SpaceJsonValue>;
    }
  | {
      op: "create-relation";
      tempId: string;
      sourceId: string;
      targetId: string;
      kind: SpaceRelationKind;
      rationale: string;
      confidence?: number;
    }
  | {
      op: "add-provenance";
      targetKind: "node" | "relation";
      targetId: string;
      sourceNodeId: string;
      locator: KnowledgeLocator;
      sourceRevision: string;
      quote: string;
    };

export interface KnowledgeChangeSet {
  spaceId: string;
  actor: string;
  operations: KnowledgeChange[];
}
