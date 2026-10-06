import { getErrorMessage } from "../../../foundation/errors/error-format";
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
    if (typeof source !== "string") await source.dispose?.();
    throw error;
  }
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
  try {
    if (!prepared.jobId)
      return await finalizePreparedImport({ context, source, position, reveal, spaceId, taskId, prepared });
    return await commitManagedImport({ context, spaceId, taskId, prepared, position, reveal, source });
  } finally {
    if (typeof source !== "string") await source.dispose?.();
  }
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
    tasks.fail(taskId, getErrorMessage(error));
    throw error;
  }
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
    context.finalizePendingContent(spaceId, nodes, reveal);
  } catch (error) {
    await Promise.all(nodes.map((node) => context.discardPendingContent(spaceId, node.id)));
    tasks.fail(taskId, getErrorMessage(error));
    throw error;
  }
  tasks.succeed(taskId);
  return nodes;
}
