import type { SpaceKnowledgeObjectType } from "../../knowledge/model/object-types";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceStructure } from "../../model/structure/structure-schema";
import type { CanvasRelationOverrideColor } from "./canvas-document";
import type { StructureLayout } from "./structure-layout";

export interface EditorFragment {
  structure: SpaceStructure;
  layout: StructureLayout;
  nodes: {
    id: string;
    kind: SpaceNodePayload["kind"];
    data: SpaceNodePayload;
    sourceChunks: readonly KnowledgeChunkSeed[];
  }[];
  relations: {
    id: string;
    sourceId: string;
    targetId: string;
    kind: SpaceRelation["kind"];
    rationale: string;
    status: "active" | "contested";
    confidence?: number;
  }[];
  objectTypes: Pick<SpaceKnowledgeObjectType, "id" | "name" | "icon" | "description" | "fields">[];
  relationStyles: { relationId: string; color: CanvasRelationOverrideColor }[];
}
export interface FragmentPlacement {
  parentId: string | null;
  delta: { x: number; y: number };
}
