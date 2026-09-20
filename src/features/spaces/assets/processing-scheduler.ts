import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { resumePendingEnrichmentJobs } from "../enrichment/job-scheduler";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { runSpaceAssetDerivation } from "../session/work-scheduler";
import { cancelSpaceAssetProcessing, deriveSpaceAssetData } from "./asset-operations";

const ASSET_PROCESSING_CANCELLED = "SPACE_ASSET_PROCESSING_CANCELLED";

export interface SpaceAssetProcessingContext {
  applyNodeDataForSpace(spaceId: string, id: string, data: SpaceNodePayload): void;
}

// oxlint-disable-next-line eslint/max-params -- Public feature API is shared with scoped-out canvas callers; preserve its contract.
export function scheduleSpaceAssetProcessing(
  context: SpaceAssetProcessingContext,
  spaceId: string,
  nodeId: string,
  data: SpaceNodePayload,
  existingTaskId?: string,
) {
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
        if (cancelRequested) throw new Error(ASSET_PROCESSING_CANCELLED);
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
    if (cancelRequested || String(error).includes(ASSET_PROCESSING_CANCELLED)) {
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
