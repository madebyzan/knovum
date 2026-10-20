import { warn as logWarn } from "@tauri-apps/plugin-log";

import { formatErrorDiagnostic, getErrorMessage } from "../../../foundation/errors/error-format";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { prepareSpaceAssetImport, type SpaceImportSourceAccess } from "./asset-operations";
import type {
  PreparedSpaceImportOperation,
  SpaceImportContext,
  SpaceImportSessionLabels,
  SpaceImportSource,
} from "./import-contract";
import { commitManagedImport, type CommitManagedImportOptions } from "./import-transfer";
import { importedSpaceNodePayload } from "./imported-asset-payload";

interface SpaceImportSourceInput {
  context: SpaceImportContext;
  spaceId: string;
  source: SpaceImportSource;
  position: { x: number; y: number };
  labels: SpaceImportSessionLabels;
  reveal?: boolean;
}

export async function importSpaceSource(input: SpaceImportSourceInput): Promise<SpaceCanvasNode[]> {
  const { context, spaceId, source, position, labels, reveal = true } = input;
  let operation: PreparedSpaceImportOperation;
  try {
    await context.ensureImportTarget(spaceId);
    operation = await prepareSpaceImportSource(source, labels, "scoped");
  } catch (error) {
    return disposeFailedSource(source, error);
  }
  return commitPreparedSpaceImport({ context, spaceId, operation, position, reveal });
}

export async function prepareSpaceImportSource(
  source: SpaceImportSource,
  labels: SpaceImportSessionLabels,
  sourceAccess: SpaceImportSourceAccess,
): Promise<PreparedSpaceImportOperation> {
  const tasks = useTaskStore();
  const taskId = crypto.randomUUID();
  tasks.restoreRunning({
    id: taskId,
    title: labels.title,
    statusMessage: labels.preparing,
    startedAt: Date.now(),
  });
  try {
    const prepared = await (typeof source === "string"
      ? prepareSpaceAssetImport(source, sourceAccess)
      : source.prepare());
    tasks.title(taskId, prepared.asset.name);
    return { source, prepared, taskId };
  } catch (error) {
    tasks.fail(taskId, getErrorMessage(error));
    throw error;
  }
}

interface PreparedSpaceImportInput {
  context: SpaceImportContext;
  spaceId: string;
  operation: PreparedSpaceImportOperation;
  position: { x: number; y: number };
  reveal?: boolean;
}

export async function commitPreparedSpaceImport(input: PreparedSpaceImportInput): Promise<SpaceCanvasNode[]> {
  const { context, operation, position, spaceId, reveal = true } = input;
  const { prepared, source, taskId } = operation;
  let nodes: SpaceCanvasNode[];
  try {
    if (!prepared.jobId)
      nodes = await finalizePreparedImport({ context, source, position, reveal, spaceId, taskId, prepared });
    else nodes = await commitManagedImport({ context, spaceId, taskId, prepared, position, reveal, source });
  } catch (error) {
    return disposeFailedSource(source, error);
  }
  await disposeCommittedSource(source);
  return nodes;
}

async function finalizePreparedImport(options: CommitManagedImportOptions) {
  const { context, source, position, reveal, spaceId, taskId, prepared } = options;
  const tasks = useTaskStore();
  let nodes: SpaceCanvasNode[] = [];
  try {
    nodes = await context.createPendingContentForSpace(
      spaceId,
      importedSpaceNodePayload(
        prepared.asset,
        typeof source === "string" ? undefined : source.externalSource,
      ),
      position,
    );
    if (typeof source !== "string") await source.finalize?.();
    context.finalizePendingContent(spaceId, nodes, { reveal });
  } catch (error) {
    tasks.fail(taskId, getErrorMessage(error));
    const cleanup = await Promise.allSettled(
      nodes.map((node) => context.discardPendingContent(spaceId, node.id)),
    );
    const cleanupErrors = cleanup
      .filter((settlement): settlement is PromiseRejectedResult => settlement.status === "rejected")
      .map(({ reason }) => reason);
    if (cleanupErrors.length)
      throw new AggregateError(
        [error, ...cleanupErrors],
        "Space import failed and pending nodes could not be removed",
        { cause: error },
      );
    throw error;
  }
  tasks.succeed(taskId);
  return nodes;
}

async function disposeFailedSource(source: SpaceImportSource, error: unknown): Promise<never> {
  if (typeof source === "string" || !source.dispose) throw error;
  try {
    await source.dispose();
  } catch (cleanupError) {
    throw new AggregateError(
      [error, cleanupError],
      "Space import failed and its source could not be released",
      {
        cause: error,
      },
    );
  }
  throw error;
}

async function disposeCommittedSource(source: SpaceImportSource) {
  if (typeof source === "string" || !source.dispose) return;
  try {
    await source.dispose();
  } catch (error) {
    void logWarn(
      `Space import committed, but its source could not be released: ${formatErrorDiagnostic(error)}`,
    );
  }
}
