import { decodeAppError } from "../../../foundation/errors/app-error";
import { errorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { refreshEnrichmentTasks } from "../enrichment/job-observation";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { cancelSpaceAssetProcessing, deriveSpaceAssetContent } from "./asset-operations";
import { runSpaceAssetProcessing } from "./concurrency-budget";

export interface SpaceAssetProcessingContext {
  applyNodeContentForSpace(spaceId: string, id: string, data: SpaceNodePayload): void;
}

export function scheduleSpaceAssetProcessing(input: {
  context: SpaceAssetProcessingContext;
  spaceId: string;
  nodeId: string;
  data: SpaceNodePayload;
  existingTaskId?: string;
}) {
  const { context, spaceId, nodeId, data, existingTaskId } = input;
  const tasks = useTaskStore();
  const jobId = crypto.randomUUID();
  let cancelRequested = false;
  let currentContent = data;
  const cancel = async () => {
    cancelRequested = true;
    await cancelSpaceAssetProcessing(jobId);
  };
  const taskId =
    existingTaskId ?? tasks.start(data.title, i18n.global.t("spaces.importExport.progress.processing"));
  tasks.setStatusMessage(taskId, i18n.global.t("spaces.importExport.progress.processing"));
  tasks.setCancellation(taskId, cancel);
  void run();

  async function run() {
    let enriched: SpaceNodePayload;
    try {
      enriched = await runSpaceAssetProcessing(async () => {
        if (cancelRequested) throw new Error("Space asset processing cancelled");
        return deriveSpaceAssetContent(spaceId, nodeId, jobId, (updated) => {
          currentContent = updated;
          context.applyNodeContentForSpace(spaceId, nodeId, updated);
        });
      });
    } catch (error) {
      await handleEnrichmentFailure(error);
      return;
    }
    currentContent = enriched;
    context.applyNodeContentForSpace(spaceId, nodeId, enriched);
    try {
      await refreshEnrichmentTasks(spaceId);
      tasks.succeed(taskId);
    } catch (error) {
      tasks.fail(taskId, errorMessage(error));
    }
  }

  async function handleEnrichmentFailure(error: unknown) {
    if (cancelRequested || decodeAppError(error)?.code === "space_asset_processing_cancelled") {
      if (isSpaceManagedAssetPayload(currentContent)) {
        context.applyNodeContentForSpace(spaceId, nodeId, { ...currentContent, processingState: undefined });
      }
      return;
    }
    if (isSpaceManagedAssetPayload(currentContent)) {
      context.applyNodeContentForSpace(spaceId, nodeId, { ...currentContent, processingState: "failed" });
    }
    try {
      await refreshEnrichmentTasks(spaceId);
      tasks.fail(taskId, errorMessage(error));
    } catch (resumeError) {
      tasks.fail(taskId, `${errorMessage(error)} · ${errorMessage(resumeError)}`);
    }
  }
}
