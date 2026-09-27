import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceEditCommand } from "../../structure/space-edit-command";
import type {
  CanvasEditorPort,
  CanvasNodePlacement,
  CanvasPoint,
  CanvasSelectionSnapshot,
  CanvasViewport,
} from "./canvas-editor-port";

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

export interface CanvasHostEvents {
  onCopyNodes(nodes: readonly CanvasNodePlacement[]): Promise<void>;
  onDeleteRelations(ids: readonly string[]): Promise<void>;
  onCreateRelation(sourceNodeId: string, targetNodeId: string): Promise<void>;
  onCreateTextNode(kind: "note" | "text", point: CanvasPoint): Promise<string | undefined>;
  onCreateSection(frame: { x: number; y: number; width: number; height: number }): Promise<void>;
  onExecuteSpaceEdit(command: SpaceEditCommand): Promise<unknown>;
  onEnterGroup(structureNodeId: string): void;
  onExitGroup(): boolean;
  onInsertText(text: string, point: CanvasPoint): Promise<void>;
  onInsertUrl(url: string, point: CanvasPoint): Promise<void>;
  onImportFiles(files: readonly CanvasFileImportRequest[]): Promise<readonly CanvasFileImportResult[]>;
  onNodeActivate(nodeId: string): Promise<void>;
  onPasteNodes(point: CanvasPoint): Promise<boolean>;
  onError(error: unknown): void;
  onHistoryTransaction(action: () => Promise<void>): Promise<void>;
  onDirectManipulationChange(active: boolean): void;
  onMutationPendingChange(pending: boolean): void;
  onReady(editor: CanvasEditorPort): void;
  onSelectionChange(selection: CanvasSelectionSnapshot): void;
  onViewportChange(viewport: CanvasViewport): void;
}

export type CanvasSurfaceEvents = CanvasHostEvents;
