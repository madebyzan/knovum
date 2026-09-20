import type { SpaceEditorDocument, CanvasNodeAppearance } from "../canvas/contract/canvas-document";
import type { CanvasPersistenceHistory } from "../canvas/contract/canvas-editor";
import type { SpaceRelation } from "../model/relations/relation";
import type { JsonCanvasEdgeV1 } from "./json-canvas-contract";

export type JsonCanvasNodeContentV1 =
  | { type: "text"; text: string }
  | { type: "file"; file: string }
  | { type: "link"; url: string }
  | { type: "group"; label?: string };

export interface JsonCanvasImportNodeV1 {
  sourceId: string;
  frame: { x: number; y: number; width: number; height: number; rotation: 0 };
  appearance?: CanvasNodeAppearance;
  fileSubpath?: string;
  parentSourceId?: string;
  zIndex: number;
  content: JsonCanvasNodeContentV1;
}

export interface JsonCanvasImportPlanV1 {
  nodes: readonly JsonCanvasImportNodeV1[];
  edges: readonly JsonCanvasEdgeV1[];
}

export interface CanvasJsonImportRequest {
  sourceDirectory: string;
  plan: JsonCanvasImportPlanV1;
  baseRevision: number;
  document: SpaceEditorDocument;
}

export type CanvasJsonImportDraft = Omit<CanvasJsonImportRequest, "baseRevision">;

export interface CanvasJsonImportResult {
  importedNodes: readonly { sourceId: string; nodeId: string }[];
  importedRelations: readonly SpaceRelation[];
  document: SpaceEditorDocument;
  revision: number;
}

export interface CanvasJsonImportCommitResult extends CanvasJsonImportResult {
  history: CanvasPersistenceHistory;
}
