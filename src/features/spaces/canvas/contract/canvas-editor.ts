import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceEditorDocument } from "./canvas-document";
import type {
  CanvasEditorPort,
  CanvasPoint,
  CanvasSelectionSnapshot,
  CanvasViewport,
} from "./canvas-editor-port";
import type {
  ExecuteSpaceEditRequest,
  ExecuteSpaceEditDraft,
  ReplaySpaceEditRequest,
  ReplaySpaceEditDraft,
} from "./editor-command";
import type { SpaceEditResult } from "./editor-result";

export type {
  CanvasEditorPort,
  CanvasLayerMove,
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
  baseDocument: SpaceEditorDocument;
  nodeIds: readonly string[];
  relationIds: readonly string[];
  baseRevision: number;
  document: SpaceEditorDocument;
}

export type CanvasDeleteSelectionDraft = Omit<CanvasDeleteSelectionRequest, "baseRevision">;

export interface CanvasSelectionTransfer {
  document: SpaceEditorDocument;
  itemIds: readonly string[];
}
export interface CanvasCutSelectionRequest extends CanvasDeleteSelectionRequest {
  transfer: CanvasSelectionTransfer;
}

export type CanvasCutSelectionDraft = Omit<CanvasCutSelectionRequest, "baseRevision">;

export interface CanvasPersistenceMutationRequest {
  baseRevision: number;
  document: SpaceEditorDocument;
}

export interface CanvasPersistenceMutationResult {
  revision: number;
}

export interface CanvasPersistenceHistory {
  undo(request: CanvasPersistenceMutationRequest): Promise<CanvasPersistenceMutationResult>;
  redo(request: CanvasPersistenceMutationRequest): Promise<CanvasPersistenceMutationResult>;
}

export interface CanvasDeleteSelectionResult {
  document: SpaceEditorDocument;
  revision: number;
  history: CanvasPersistenceHistory;
}

export interface CanvasGestureCommitRequest {
  baseDocument: SpaceEditorDocument;
  baseRevision: number;
  document: SpaceEditorDocument;
}

export type CanvasGestureCommitDraft = Omit<CanvasGestureCommitRequest, "baseRevision">;

export interface CanvasGestureCommitResult {
  document: SpaceEditorDocument;
  revision: number;
  history: CanvasPersistenceHistory;
}

export interface CanvasHostEvents {
  onExecuteSpaceEdit(request: ExecuteSpaceEditRequest): Promise<SpaceEditResult>;
  onReplaySpaceEdit(request: ReplaySpaceEditRequest): Promise<SpaceEditResult>;
  onCommitGesture(request: CanvasGestureCommitRequest): Promise<CanvasGestureCommitResult>;
  onCopyNodes(transfer: CanvasSelectionTransfer): Promise<void>;
  onPrepareStructureEdit(): Promise<void>;
  onStructureToolFinished(): void;
  onCreateRelation(sourceNodeId: string, targetNodeId: string): Promise<void>;
  onCreateTextNode(kind: "note" | "text", point: CanvasPoint): Promise<string | undefined>;
  onCutSelection(request: CanvasCutSelectionRequest): Promise<CanvasDeleteSelectionResult>;
  onDeleteSelection(request: CanvasDeleteSelectionRequest): Promise<CanvasDeleteSelectionResult>;
  onDuplicateNodes(transfer: CanvasSelectionTransfer): Promise<void>;
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
  "onCommitGesture" | "onCutSelection" | "onDeleteSelection" | "onExecuteSpaceEdit" | "onReplaySpaceEdit"
> {
  onExecuteSpaceEdit(request: ExecuteSpaceEditDraft): Promise<SpaceEditResult>;
  onReplaySpaceEdit(request: ReplaySpaceEditDraft): Promise<SpaceEditResult>;
  onCommitGesture(request: CanvasGestureCommitDraft): Promise<CanvasGestureCommitResult>;
  onCutSelection(request: CanvasCutSelectionDraft): Promise<CanvasDeleteSelectionResult>;
  onDeleteSelection(request: CanvasDeleteSelectionDraft): Promise<CanvasDeleteSelectionResult>;
  onHistoryTransition(
    document: SpaceEditorDocument,
    transition: CanvasPersistenceHistory["undo"],
    baseDocument: SpaceEditorDocument,
  ): Promise<CanvasPersistenceMutationResult & { document: SpaceEditorDocument }>;
}
