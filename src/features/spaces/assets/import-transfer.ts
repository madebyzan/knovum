import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { runSpaceAssetTransfer } from "../session/work-scheduler";
import {
  cancelSpaceAssetImport,
  copyStagedSpaceAsset,
  removeSpaceAsset,
  type PreparedSpaceImport,
} from "./asset-operations";
import { formatOptionalBytes } from "../../../foundation/format/bytes";
import type { SpaceImportContext, SpaceImportSource } from "./import-contract";
import { importedSpaceNodePayload } from "./imported-asset-payload";
import { scheduleSpaceAssetProcessing } from "./processing-scheduler";

interface CommitManagedImportOptions {
  context: SpaceImportContext;
  spaceId: string;
  taskId: string;
  prepared: PreparedSpaceImport;
  position: { x: number; y: number };
  reveal: boolean;
  source: SpaceImportSource;
}

export async function commitManagedImport(options: CommitManagedImportOptions) {
  const { context, spaceId, taskId, prepared, position, reveal, source } = options;
  const tasks = useTaskStore();
  const jobId = prepared.jobId!;
  let cancelRequested = false;
  let node: SpaceCanvasNode | undefined;
  let copiedPath: string | undefined;
  let imported: { node: SpaceCanvasNode; data: SpaceNodePayload };
  tasks.setCancellation(taskId, async () => {
    cancelRequested = true;
    await cancelSpaceAssetImport(jobId);
  });
  try {
    [node] = await context.createPendingItemForSpace(
      spaceId,
      importedSpaceNodePayload(prepared.asset),
      position,
    );
    if (!node) throw new Error("Space import did not create a node");
    const copied = await runSpaceAssetTransfer(() => {
      if (cancelRequested) throw new Error("Space import cancelled");
      return copyStagedSpaceAsset(jobId, (progress) => updateCopyProgress(taskId, progress));
    });
    if (cancelRequested) throw new Error("Space import cancelled");
    copiedPath = copied.path;
    const copiedData = importedSpaceNodePayload(copied);
    await context.savePendingNodeDataForSpace(spaceId, node.id, copiedData);
    node.data = copiedData;
    if (typeof source !== "string") await source.finalize();
    context.finalizePendingItems(spaceId, [node], reveal);
    imported = { node, data: copiedData };
  } catch (error) {
    await cancelSpaceAssetImport(jobId);
    if (node) await context.discardPendingItem(spaceId, node.id);
    if (copiedPath) await removeSpaceAsset(copiedPath);
    if (cancelRequested) return [];
    tasks.fail(taskId, String(error));
    throw error;
  }
  if (isSpaceManagedAssetPayload(imported.data) && imported.data.processingState === "processing") {
    scheduleSpaceAssetProcessing(context, spaceId, imported.node.id, imported.data, taskId);
  } else {
    tasks.succeed(taskId);
  }
  return [imported.node];
}

function updateCopyProgress(taskId: string, progress: { bytesCopied: number; totalBytes?: number }) {
  const tasks = useTaskStore();
  if (!progress.totalBytes) {
    tasks.detail(taskId, formatOptionalBytes(progress.bytesCopied) ?? "");
    return;
  }
  const current = formatOptionalBytes(progress.bytesCopied);
  const total = formatOptionalBytes(progress.totalBytes);
  tasks.progress(taskId, (progress.bytesCopied / progress.totalBytes) * 100, `${current} / ${total}`);
}
