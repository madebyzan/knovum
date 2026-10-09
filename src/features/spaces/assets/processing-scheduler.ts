import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { refreshEnrichmentJobs } from "../enrichment/enrichment-jobs-query";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { cancelSpaceAssetProcessing, deriveSpaceAssetContent } from "./asset-operations";
import { runSpaceAssetProcessing } from "./concurrency-budget";

export interface SpaceAssetProcessingContext {
  applyNodeContentForSpace(spaceId: string, nodeId: string, payload: SpaceNodePayload): void;
}

interface SpaceAssetProcessingInput {
  context: SpaceAssetProcessingContext;
  spaceId: string;
  nodeId: string;
  data: SpaceNodePayload;
  existingTaskId?: string;
}

export function scheduleSpaceAssetProcessing(input: SpaceAssetProcessingInput) {
  const { context, spaceId, nodeId, data, existingTaskId } = input;
  const tasks = useTaskStore();
  const jobId = crypto.randomUUID();
  let currentContent = data;
  const cancel = () => cancelSpaceAssetProcessing(jobId);
  const taskId = existingTaskId ?? jobId;
  if (existingTaskId) tasks.setCancellation(taskId, cancel);
  else
    tasks.restoreRunning(
      {
        id: taskId,
        title: data.title,
        statusMessage: i18n.global.t("spaces.importExport.progress.processing"),
        startedAt: Date.now(),
      },
      cancel,
    );
  tasks.setStatusMessage(taskId, i18n.global.t("spaces.importExport.progress.processing"));
  void run();

  async function run() {
    let enriched: SpaceNodePayload;
    try {
      enriched = await runSpaceAssetProcessing(() =>
        deriveSpaceAssetContent({
          spaceId,
          nodeId,
          jobId,
          onUpdate: (updated) => {
            currentContent = updated;
            context.applyNodeContentForSpace(spaceId, nodeId, updated);
          },
        }),
      );
    } catch (error) {
      await handleEnrichmentFailure(error);
      return;
    }
    currentContent = enriched;
    context.applyNodeContentForSpace(spaceId, nodeId, enriched);
    tasks.succeed(taskId);
    void refreshEnrichmentJobs();
  }

  async function handleEnrichmentFailure(error: unknown) {
    if (decodeAppError(error)?.code === "space_asset_processing_cancelled") {
      if (isSpaceManagedAssetPayload(currentContent)) {
        context.applyNodeContentForSpace(spaceId, nodeId, { ...currentContent, processingState: undefined });
      }
      tasks.cancelled(taskId);
      return;
    }
    if (isSpaceManagedAssetPayload(currentContent)) {
      context.applyNodeContentForSpace(spaceId, nodeId, { ...currentContent, processingState: "failed" });
    }
    tasks.fail(taskId, getErrorMessage(error));
    void refreshEnrichmentJobs();
  }
}
