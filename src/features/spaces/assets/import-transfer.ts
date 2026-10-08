import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { PreparedSpaceImport } from "../contracts/generated/PreparedSpaceImport";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { cancelSpaceAssetImport, copyStagedSpaceAsset, removeSpaceAsset } from "./asset-operations";
import { runSpaceAssetTransfer } from "./concurrency-budget";
import { formatByteSize } from "./format-byte-size";
import type { SpaceImportContext, SpaceImportSource } from "./import-contract";
import { importedSpaceNodePayload } from "./imported-asset-payload";
import { scheduleSpaceAssetProcessing } from "./processing-scheduler";

export interface CommitManagedImportOptions {
  context: SpaceImportContext;
  spaceId: string;
  taskId: string;
  prepared: PreparedSpaceImport;
  position: { x: number; y: number };
  reveal: boolean;
  source: SpaceImportSource;
}

interface ManagedImportProgress {
  node: SpaceCanvasNode | undefined;
  copiedPath: string | undefined;
}

export async function commitManagedImport(options: CommitManagedImportOptions) {
  const tasks = useTaskStore();
  const progress: ManagedImportProgress = { node: undefined, copiedPath: undefined };
  let imported: { node: SpaceCanvasNode; data: SpaceNodePayload };
  tasks.setCancellation(options.taskId, () => cancelSpaceAssetImport(options.prepared.jobId!));
  try {
    imported = await performManagedImport(options, progress);
  } catch (error) {
    return settleFailedImport(options, progress, error);
  }
  completeImportProcessing(options, imported);
  return [imported.node];
}

async function performManagedImport(options: CommitManagedImportOptions, progress: ManagedImportProgress) {
  const { context, spaceId, prepared, position, reveal, source, taskId } = options;
  const externalSource = typeof source === "string" ? undefined : source.externalSource;
  const [node] = await context.createPendingContentForSpace(
    spaceId,
    importedSpaceNodePayload(prepared.asset, externalSource),
    position,
  );
  if (!node) throw new Error("Space import did not create a node");
  progress.node = node;
  const copied = await runSpaceAssetTransfer(() =>
    copyStagedSpaceAsset(prepared.jobId!, (copyProgress) => updateCopyProgress(taskId, copyProgress)),
  );
  progress.copiedPath = copied.path;
  const data = importedSpaceNodePayload(copied, externalSource);
  await context.savePendingNodeContentForSpace(spaceId, {
    id: node.id,
    expectedData: node.data,
    data,
  });
  node.data = data;
  if (typeof source !== "string") await source.finalize?.();
  context.finalizePendingContent(spaceId, [node], reveal);
  return { node, data };
}

async function settleFailedImport(
  options: CommitManagedImportOptions,
  progress: ManagedImportProgress,
  error: unknown,
): Promise<SpaceCanvasNode[]> {
  const { context, spaceId, taskId, prepared } = options;
  const tasks = useTaskStore();
  const cancelled = decodeAppError(error)?.code === "space_asset_import_cancelled";
  if (cancelled) tasks.cancelled(taskId);
  else tasks.fail(taskId, getErrorMessage(error));
  const cleanup = await Promise.allSettled([
    cancelSpaceAssetImport(prepared.jobId!),
    ...(progress.node ? [context.discardPendingContent(spaceId, progress.node.id)] : []),
    ...(progress.copiedPath ? [removeSpaceAsset(progress.copiedPath)] : []),
  ]);
  const cleanupErrors = cleanup
    .filter((settlement): settlement is PromiseRejectedResult => settlement.status === "rejected")
    .map(({ reason }) => reason);
  if (cleanupErrors.length)
    throw new AggregateError(
      [error, ...cleanupErrors],
      "Space import failed and pending resources could not be removed",
      { cause: error },
    );
  if (cancelled) return [];
  throw error;
}

function completeImportProcessing(
  options: CommitManagedImportOptions,
  imported: { node: SpaceCanvasNode; data: SpaceNodePayload },
) {
  const { context, spaceId, taskId } = options;
  const tasks = useTaskStore();
  if (isSpaceManagedAssetPayload(imported.data) && imported.data.processingState === "processing") {
    scheduleSpaceAssetProcessing({
      context,
      spaceId,
      nodeId: imported.node.id,
      data: imported.data,
      existingTaskId: taskId,
    });
  } else {
    tasks.succeed(taskId);
  }
}

function updateCopyProgress(taskId: string, progress: { bytesCopied: number; totalBytes?: number }) {
  const tasks = useTaskStore();
  if (!progress.totalBytes) {
    tasks.setStatusMessage(taskId, formatByteSize(progress.bytesCopied) ?? "");
    return;
  }
  const current = formatByteSize(progress.bytesCopied);
  const total = formatByteSize(progress.totalBytes);
  tasks.progress(taskId, (progress.bytesCopied / progress.totalBytes) * 100, `${current} / ${total}`);
}
