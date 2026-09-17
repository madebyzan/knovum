import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasDocumentV1 } from "./canvas-document";
import type {
  CanvasEditorPort,
  CanvasNodePlacement,
  CanvasPoint,
  CanvasSelectionSnapshot,
  CanvasViewport,
} from "./canvas-editor-port";
import type { SectionAssignment } from "./section-membership";

export type {
  CanvasEditorPort,
  CanvasLayerMove,
  CanvasNodePlacement,
  CanvasPoint,
  CanvasSelectionSnapshot,
  CanvasViewport,
} from "./canvas-editor-port";

export interface CanvasFileImportRequest {
  sourceId: string;
  path: string;
  position: CanvasPoint;
}

export interface CanvasFileImportResult {
  node: SpaceCanvasNode;
  sourceId: string;
}

export interface CanvasHistoryEntry {
  undo(): Promise<void>;
  redo(): Promise<void>;
}

export interface CanvasDeleteSelectionRequest {
  nodeIds: readonly string[];
  relationIds: readonly string[];
  baseRevision: number;
  document: CanvasDocumentV1;
}

export type CanvasDeleteSelectionDraft = Omit<CanvasDeleteSelectionRequest, "baseRevision">;

export interface CanvasCutSelectionRequest extends CanvasDeleteSelectionRequest {
  nodePlacements: readonly CanvasNodePlacement[];
}

export type CanvasCutSelectionDraft = Omit<CanvasCutSelectionRequest, "baseRevision">;

export interface CanvasPersistenceMutationRequest {
  baseRevision: number;
  document: CanvasDocumentV1;
}

export interface CanvasPersistenceMutationResult {
  revision: number;
}

export interface CanvasPersistenceHistory {
  undo(request: CanvasPersistenceMutationRequest): Promise<CanvasPersistenceMutationResult>;
  redo(request: CanvasPersistenceMutationRequest): Promise<CanvasPersistenceMutationResult>;
}

export interface CanvasDeleteSelectionResult {
  document: CanvasDocumentV1;
  revision: number;
  history: CanvasPersistenceHistory;
}

export interface CanvasGestureCommitRequest {
  assignments: readonly SectionAssignment[];
  baseRevision: number;
  document: CanvasDocumentV1;
}

export type CanvasGestureCommitDraft = Omit<CanvasGestureCommitRequest, "baseRevision">;

export interface CanvasGestureCommitResult {
  document: CanvasDocumentV1;
  revision: number;
  history: CanvasPersistenceHistory;
}

export interface CanvasHostEvents {
  onCommitGesture(request: CanvasGestureCommitRequest): Promise<CanvasGestureCommitResult>;
  onCopyNodes(nodes: readonly CanvasNodePlacement[]): Promise<void>;
  onCreateRelation(sourceNodeId: string, targetNodeId: string): Promise<void>;
  onCreateTextNode(kind: "note" | "text", point: CanvasPoint): Promise<string | undefined>;
  onCutSelection(request: CanvasCutSelectionRequest): Promise<CanvasDeleteSelectionResult>;
  onDeleteSelection(request: CanvasDeleteSelectionRequest): Promise<CanvasDeleteSelectionResult>;
  onDuplicateNodes(nodes: readonly CanvasNodePlacement[]): Promise<void>;
  onInsertText(text: string, point: CanvasPoint): Promise<void>;
  onInsertUrl(url: string, point: CanvasPoint): Promise<void>;
  onImportFiles(files: readonly CanvasFileImportRequest[]): Promise<readonly CanvasFileImportResult[]>;
  onNodeActivate(nodeId: string): Promise<void>;
  onPasteNodes(point: CanvasPoint): Promise<boolean>;
  onError(error: unknown): void;
  onHistoryEntry(entry: CanvasHistoryEntry): void;
  onHistoryTransaction(action: () => Promise<void>): Promise<void>;
  onDirectManipulationChange(active: boolean): void;
  onMutationPendingChange(pending: boolean): void;
  onReady(editor: CanvasEditorPort): void;
  onSelectionChange(selection: CanvasSelectionSnapshot): void;
  onViewportChange(viewport: CanvasViewport): void;
}

export interface CanvasSurfaceEvents extends Omit<
  CanvasHostEvents,
  "onCommitGesture" | "onCutSelection" | "onDeleteSelection"
> {
  onCommitGesture(request: CanvasGestureCommitDraft): Promise<CanvasGestureCommitResult>;
  onCutSelection(request: CanvasCutSelectionDraft): Promise<CanvasDeleteSelectionResult>;
  onDeleteSelection(request: CanvasDeleteSelectionDraft): Promise<CanvasDeleteSelectionResult>;
  onHistoryTransition(
    document: CanvasDocumentV1,
    transition: CanvasPersistenceHistory["undo"],
  ): Promise<CanvasPersistenceMutationResult & { document: CanvasDocumentV1 }>;
}
