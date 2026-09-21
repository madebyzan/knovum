import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceRelation } from "../model/relations/relation";
import type { StructureLayout } from "../model/structure/structure-contract";
import type { JsonCanvasEdgeV1 } from "./json-canvas-contract";

type ContentAppearance = Extract<StructureLayout, { kind: "content" }>["appearance"];
type SectionAppearance = Extract<StructureLayout, { kind: "section" }>["appearance"];

export type JsonCanvasNodeContentV1 =
  | { type: "text"; text: string }
  | { type: "file"; file: string }
  | { type: "link"; url: string };

export interface JsonCanvasImportContentV1 {
  sourceId: string;
  frame: { x: number; y: number; width: number; height: number };
  appearance?: ContentAppearance;
  fileSubpath?: string;
  parentSectionSourceId?: string;
  order: number;
  content: JsonCanvasNodeContentV1;
}

export interface JsonCanvasImportSectionV1 {
  sourceId: string;
  frame: { x: number; y: number; width: number; height: number };
  title?: string;
  appearance?: SectionAppearance;
  order: number;
}

export interface JsonCanvasImportPlanV1 {
  contents: readonly JsonCanvasImportContentV1[];
  sections: readonly JsonCanvasImportSectionV1[];
  edges: readonly JsonCanvasEdgeV1[];
}

export interface CanvasJsonImportRequest {
  sourceDirectory: string;
  plan: JsonCanvasImportPlanV1;
  baseRevision: number;
}

export type CanvasJsonImportDraft = Omit<CanvasJsonImportRequest, "baseRevision">;

export interface CanvasJsonImportResult {
  importedNodes: readonly { sourceId: string; nodeId: string }[];
  importedRelations: readonly SpaceRelation[];
  snapshot: SpaceEditSnapshot;
  revision: number;
}

export type CanvasJsonImportCommitResult = CanvasJsonImportResult;
