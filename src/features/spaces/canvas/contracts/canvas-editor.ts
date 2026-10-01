import type { SpaceEditCommand } from "../../contracts/generated/SpaceEditCommand";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type {
  CanvasEditorPort,
  CanvasNodePlacement,
  CanvasPoint,
  CanvasRect,
  CanvasSelectionSnapshot,
  CanvasViewport,
} from "./canvas-editor-port";

export interface CanvasFileImportRequest {
  sourceId: string;
  path: string;
  position: CanvasPoint;
}

export interface CanvasFileImportReceipt {
  node: SpaceCanvasNode;
  sourceId: string;
}

export interface CanvasHostEvents {
  onCopyNodes(nodes: readonly CanvasNodePlacement[]): Promise<void>;
  onDeleteRelations(ids: readonly string[]): Promise<void>;
  onCreateRelation(sourceNodeId: string, targetNodeId: string): Promise<void>;
  onCreateTextNode(kind: "note" | "text", point: CanvasPoint): Promise<string | undefined>;
  onCreateSection(frame: CanvasRect): Promise<void>;
  onExecuteSpaceEdit(command: SpaceEditCommand): Promise<unknown>;
  onEnterGroup(structureNodeId: string): void;
  onExitGroup(): boolean;
  onInsertText(text: string, point: CanvasPoint): Promise<void>;
  onInsertUrl(url: string, point: CanvasPoint): Promise<void>;
  onImportFiles(files: readonly CanvasFileImportRequest[]): Promise<readonly CanvasFileImportReceipt[]>;
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
