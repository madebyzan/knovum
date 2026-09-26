import { i18n } from "../../../foundation/i18n/i18n";
import { decodeCommandError } from "../../../foundation/ipc/command-error";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { resumePendingEnrichmentJobs } from "../enrichment/job-scheduler";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { runSpaceAssetDerivation } from "../session/actions/work-scheduler";
import { cancelSpaceAssetProcessing, deriveSpaceAssetData } from "./asset-operations";

export interface SpaceAssetProcessingContext {
  applyNodeDataForSpace(spaceId: string, id: string, data: SpaceNodePayload): void;
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
  let currentData = data;
  const cancel = async () => {
    cancelRequested = true;
    await cancelSpaceAssetProcessing(jobId);
  };
  const taskId = existingTaskId ?? tasks.start(data.title, i18n.global.t("spaces.transfer.importProcessing"));
  tasks.detail(taskId, i18n.global.t("spaces.transfer.importProcessing"));
  tasks.setCancellation(taskId, cancel);
  void run();

  async function run() {
    let enriched: SpaceNodePayload;
    try {
      enriched = await runSpaceAssetDerivation(async () => {
        if (cancelRequested) throw new Error("Space asset processing cancelled");
        return deriveSpaceAssetData(spaceId, nodeId, jobId, (updated) => {
          currentData = updated;
          context.applyNodeDataForSpace(spaceId, nodeId, updated);
        });
      });
    } catch (error) {
      await handleEnrichmentFailure(error);
      return;
    }
    currentData = enriched;
    context.applyNodeDataForSpace(spaceId, nodeId, enriched);
    try {
      await resumePendingEnrichmentJobs(context, spaceId);
      tasks.succeed(taskId);
    } catch (error) {
      tasks.fail(taskId, String(error));
    }
  }

  async function handleEnrichmentFailure(error: unknown) {
    if (cancelRequested || decodeCommandError(error)?.code === "space_asset_processing_cancelled") {
      if (isSpaceManagedAssetPayload(currentData)) {
        context.applyNodeDataForSpace(spaceId, nodeId, { ...currentData, processingState: undefined });
      }
      return;
    }
    if (isSpaceManagedAssetPayload(currentData)) {
      context.applyNodeDataForSpace(spaceId, nodeId, { ...currentData, processingState: "failed" });
    }
    try {
      await resumePendingEnrichmentJobs(context, spaceId);
      tasks.fail(taskId, String(error));
    } catch (resumeError) {
      tasks.fail(taskId, `${String(error)} · ${String(resumeError)}`);
    }
  }
}
