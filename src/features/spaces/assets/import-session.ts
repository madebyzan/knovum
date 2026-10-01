import { errorMessage } from "../../../foundation/errors/error-format";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import {
  prepareSpaceAssetImport,
  type PreparedSpaceImport,
  type SpaceImportSourceAccess,
} from "./asset-operations";
import type {
  PreparedSpaceImportOperation,
  SpaceImportContext,
  SpaceImportSessionLabels,
  SpaceImportSource,
} from "./import-contract";
import { commitManagedImport } from "./import-transfer";
import { importedSpaceNodePayload } from "./imported-asset-payload";

export async function importSpaceSource(input: {
  context: SpaceImportContext;
  spaceId: string;
  source: SpaceImportSource;
  position: { x: number; y: number };
  labels: SpaceImportSessionLabels;
  reveal?: boolean;
}): Promise<SpaceCanvasNode[]> {
  const { context, spaceId, source, position, labels, reveal = true } = input;
  await context.ensureImportTarget(spaceId);
  const operation = await prepareSpaceImportSource(source, labels, "scoped");
  return commitPreparedSpaceImport({ context, spaceId, operation, position, reveal });
}

export async function prepareSpaceImportSource(
  source: SpaceImportSource,
  labels: SpaceImportSessionLabels,
  sourceAccess: SpaceImportSourceAccess,
): Promise<PreparedSpaceImportOperation> {
  const tasks = useTaskStore();
  const taskId = tasks.start(labels.title, labels.preparing);
  const prepared = await prepareImportSource(source, sourceAccess, tasks, taskId);
  return { source, prepared, taskId };
}

export function commitPreparedSpaceImport(input: {
  context: SpaceImportContext;
  spaceId: string;
  operation: PreparedSpaceImportOperation;
  position: { x: number; y: number };
  reveal?: boolean;
}): Promise<SpaceCanvasNode[]> {
  const { context, operation, position, spaceId, reveal = true } = input;
  const { prepared, source, taskId } = operation;
  if (!prepared.jobId)
    return finalizePreparedImport({ context, source, position, reveal, spaceId, taskId, prepared });
  return commitManagedImport({ context, spaceId, taskId, prepared, position, reveal, source });
}

async function prepareImportSource(
  source: SpaceImportSource,
  sourceAccess: SpaceImportSourceAccess,
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
) {
  try {
    const prepared = await (typeof source === "string"
      ? prepareSpaceAssetImport(source, sourceAccess)
      : source.prepare());
    tasks.title(taskId, prepared.asset.name);
    return prepared;
  } catch (error) {
    tasks.fail(taskId, errorMessage(error));
    throw error;
  }
}

interface FinalizePreparedImportOptions {
  context: SpaceImportContext;
  source: SpaceImportSource;
  position: { x: number; y: number };
  reveal: boolean;
  spaceId: string;
  taskId: string;
  prepared: PreparedSpaceImport;
}

async function finalizePreparedImport(options: FinalizePreparedImportOptions) {
  const { context, source, position, reveal, spaceId, taskId, prepared } = options;
  const tasks = useTaskStore();
  let nodes: SpaceCanvasNode[] = [];
  try {
    nodes = await context.createPendingContentForSpace(
      spaceId,
      importedSpaceNodePayload(prepared.asset),
      position,
    );
    if (typeof source !== "string") await source.finalize();
    context.finalizePendingContent(spaceId, nodes, reveal);
  } catch (error) {
    await Promise.all(nodes.map((node) => context.discardPendingContent(spaceId, node.id)));
    tasks.fail(taskId, errorMessage(error));
    throw error;
  }
  tasks.succeed(taskId);
  return nodes;
}
