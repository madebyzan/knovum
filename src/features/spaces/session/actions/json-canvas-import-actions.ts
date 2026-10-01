import { i18n } from "../../../../foundation/i18n/i18n";
import { decodeCommandError } from "../../../../foundation/ipc/command-error";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { scheduleSpaceAssetProcessing } from "../../assets/processing-scheduler";
import type {
  CanvasImportReceipt,
  CanvasJsonImportRequest,
} from "../../canvas-exchange/json-canvas-import-contract";
import {
  cancelJsonCanvasImport,
  importJsonCanvas as persistJsonCanvas,
} from "../../canvas-exchange/json-canvas-import-operations";
import type { SpacePlacement } from "../../canvas/navigation-requests";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import type { SpaceState } from "../session-state";
import { loadSpaceSessionProjection } from "../space-projection";
import type { createSpaceMutations } from "./node-mutations";

interface JsonCanvasImportContext {
  ensureActiveSpacePersisted(): Promise<void>;
  mutations: Pick<
    ReturnType<typeof createSpaceMutations>,
    "commitPendingContent" | "applyNodeContentForSpace"
  >;
  placement: Pick<SpacePlacement, "requestReveal">;
  state: SpaceState;
}

export function createJsonCanvasImportActions(context: JsonCanvasImportContext) {
  async function importJsonCanvas(request: CanvasJsonImportRequest): Promise<CanvasImportReceipt> {
    const spaceId = context.state.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active to import JSON Canvas content");
    await context.ensureActiveSpacePersisted();
    if (context.state.activeSpaceId.value !== spaceId) throw new Error("JSON Canvas import target changed");
    const tasks = useTaskStore();
    const taskId = tasks.start(i18n.global.t("spaces.importExport.progress.resource"));
    const mutationId = crypto.randomUUID();
    const operation = executeImport(spaceId);
    return operation;

    async function cancelImport() {
      await cancelJsonCanvasImport(mutationId);
      await operation.catch((error: unknown) => {
        if (decodeCommandError(error)?.code !== "json_canvas_import_cancelled") throw error;
      });
    }

    async function executeImport(targetSpaceId: string) {
      try {
        const committed = await persistJsonCanvas(
          {
            sourcePath: request.sourcePath,
            baseRevision: request.baseRevision,
            spaceId: targetSpaceId,
            mutationId,
            noteTitle: i18n.global.t("spaces.editor.note.new"),
            sectionTitle: i18n.global.t("spaces.canvas.sections.untitled"),
          },
          (progress) => {
            if (progress.stage === "preparing") tasks.setCancellation(taskId, cancelImport);
            if (progress.totalBytes)
              tasks.progress(taskId, (progress.bytesCopied / progress.totalBytes) * 100);
          },
        );
        tasks.setCancellation(taskId);
        const projection = await loadSpaceSessionProjection(targetSpaceId);
        const ids = new Set(committed.importedNodes.map(({ nodeId }) => nodeId));
        const imported = projection.nodes.filter(({ id }) => ids.has(id));
        if (context.state.activeSpaceId.value === targetSpaceId) {
          context.state.nodes.value = projection.nodes;
          context.state.relations.value = projection.relations;
          context.state.editSnapshot.value = projection.editSnapshot;
          context.mutations.commitPendingContent(targetSpaceId, imported);
          context.placement.requestReveal(imported.map(({ id }) => id));
        }
        for (const node of imported) {
          if (isSpaceManagedAssetPayload(node.data))
            scheduleSpaceAssetProcessing({
              context: context.mutations,
              spaceId: targetSpaceId,
              nodeId: node.id,
              data: node.data,
            });
        }
        tasks.succeed(taskId);
        return {
          ...committed,
          importedRelations: projection.relations.filter(
            ({ sourceId, targetId }) => ids.has(sourceId) && ids.has(targetId),
          ),
          snapshot: projection.editSnapshot,
        };
      } catch (error) {
        if (decodeCommandError(error)?.code === "json_canvas_import_cancelled") tasks.cancelled(taskId);
        else tasks.fail(taskId, String(error));
        throw error;
      }
    }
  }
  return { importJsonCanvas };
}
