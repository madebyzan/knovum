import type { ExternalContentSource } from "../contracts/generated/ExternalContentSource";
import type { PreparedSpaceImport } from "../contracts/generated/PreparedSpaceImport";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../nodes/node-operations";
import type { FinalizePendingContentOptions } from "../session/actions/placement-actions";
import type { SpaceAssetProcessingContext } from "./processing-scheduler";

export interface PendingContentContext {
  createPendingContentForSpace(
    spaceId: string,
    data: SpaceNodePayload,
    position?: { x: number; y: number },
  ): Promise<SpaceCanvasNode[]>;
  savePendingNodeContentForSpace(spaceId: string, change: NodePayloadChangeInput): Promise<void>;
  finalizePendingContent(
    spaceId: string,
    contentNodes: readonly SpaceCanvasNode[],
    options?: FinalizePendingContentOptions,
  ): void;
  discardPendingContent(spaceId: string, nodeId: string): Promise<void>;
}

export interface SpaceImportContext extends SpaceAssetProcessingContext, PendingContentContext {
  ensureImportTarget(spaceId: string): Promise<void>;
}

export interface SpaceImportSessionLabels {
  title: string;
  preparing: string;
}

interface PreparedSpaceImportSource {
  externalSource?: ExternalContentSource;
  dispose?(): Promise<void>;
  prepare(): Promise<PreparedSpaceImport>;
  finalize?(): Promise<void>;
}

export type SpaceImportSource = string | PreparedSpaceImportSource;

export interface PreparedSpaceImportOperation {
  source: SpaceImportSource;
  prepared: PreparedSpaceImport;
  taskId: string;
}
