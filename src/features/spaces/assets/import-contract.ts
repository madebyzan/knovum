import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import type { PreparedSpaceImport } from "./asset-operations";
import type { SpaceAssetProcessingContext } from "./processing-scheduler";

export interface SpaceImportContext extends SpaceAssetProcessingContext {
  ensureImportTarget(spaceId: string): Promise<void>;
  createPendingContentForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position?: { x: number; y: number },
  ): Promise<SpaceCanvasNode[]>;
  savePendingNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<void>;
  finalizePendingContent(spaceId: string, contentNodes: readonly SpaceCanvasNode[], reveal?: boolean): void;
  discardPendingContent(spaceId: string, id: string): Promise<void>;
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
