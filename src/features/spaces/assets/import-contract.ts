import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { PreparedSpaceImport } from "./asset-operations";
import type { SpaceAssetProcessingContext } from "./processing-scheduler";

export interface SpaceImportContext extends SpaceAssetProcessingContext {
  ensureImportTarget(spaceId: string): Promise<void>;
  createPendingItemForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position?: { x: number; y: number },
  ): Promise<SpaceCanvasNode[]>;
  savePendingNodeDataForSpace(spaceId: string, id: string, data: SpaceNodePayload): Promise<void>;
  finalizePendingItems(spaceId: string, items: readonly SpaceCanvasNode[], reveal?: boolean): void;
  discardPendingItem(spaceId: string, id: string): Promise<void>;
}

export interface SpaceImportSessionLabels {
  title: string;
  preparing: string;
}

export interface PreparedSpaceImportSource {
  prepare(): Promise<PreparedSpaceImport>;
  finalize(): Promise<void>;
}

export type SpaceImportSource = string | PreparedSpaceImportSource;

export interface PreparedSpaceImportOperation {
  source: SpaceImportSource;
  prepared: PreparedSpaceImport;
  taskId: string;
}
