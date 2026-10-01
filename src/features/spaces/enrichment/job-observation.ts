import { listen } from "@tauri-apps/api/event";
import { onScopeDispose, type Ref } from "vue";

import { errorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { EnrichmentJobUpdate } from "../contracts/generated/EnrichmentJobUpdate";
import {
  appendSpaceMediaAnalysisPreview,
  clearSpaceMediaAnalysisPreview,
  type SpaceMediaAnalysisPreviews,
} from "../media/analysis-preview";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { cancelEnrichmentJob, listEnrichmentJobs, retryEnrichmentJob } from "./enrichment-jobs";
import { enrichmentJobTitle, failedEnrichmentTask, runningEnrichmentTask } from "./job-tasks";

export interface SpaceEnrichmentObserverContext {
  activeSpaceId: Ref<string | undefined>;
  analysisIds: Ref<string[]>;
  analysisPreviews: Ref<SpaceMediaAnalysisPreviews>;
  applyNodeContentForSpace(spaceId: string, id: string, data: SpaceNodePayload): void;
}

export function observeEnrichmentJobs(context: SpaceEnrichmentObserverContext) {
  let disposed = false;
  const subscription = listen<EnrichmentJobUpdate>("space-enrichment-update", ({ payload }) => {
    const tasks = useTaskStore();
    projectMediaProgress(context, payload);
    tasks.restoreRunning(
      { id: payload.jobId, title: enrichmentJobTitle(payload.jobType), startedAt: payload.createdAt },
      () => cancelEnrichmentJob(payload.jobId),
    );
    if (payload.progress) tasks.progress(payload.jobId, payload.progress.progress, payload.progress.stage);
    if (payload.data)
      context.applyNodeContentForSpace(
        payload.spaceId,
        payload.nodeId,
        validateSpaceNodePayload(payload.data),
      );
    if (payload.status === "succeeded") tasks.succeed(payload.jobId);
    else if (payload.status === "cancelled") tasks.cancelled(payload.jobId);
    else if (payload.status === "failed") {
      if (!payload.error) throw new Error("Failed enrichment event is missing its error");
      tasks.fail(payload.jobId, payload.error);
      tasks.setObservedRetry(payload.jobId, () => retryEnrichmentJob(payload.jobId));
    }
  });
  const failure = listen<string>("space-enrichment-error", ({ payload }) => {
    const tasks = useTaskStore();
    tasks.start(
      i18n.global.t("spaces.processing.queue.title"),
      undefined,
      undefined,
      "space-enrichment-scheduler",
    );
    tasks.fail("space-enrichment-scheduler", payload);
  });
  for (const registration of [subscription, failure]) {
    void registration
      .then((unlisten) => {
        return disposed ? unlisten() : undefined;
      })
      .catch(reportSubscriptionFailure);
  }
  onScopeDispose(() => {
    disposed = true;
    for (const registration of [subscription, failure])
      void registration.then((unlisten) => unlisten()).catch(reportSubscriptionFailure);
  });
}

export async function refreshEnrichmentTasks(spaceId: string | undefined) {
  if (!spaceId) return;
  const tasks = useTaskStore();
  for (const job of await listEnrichmentJobs(spaceId)) {
    if (job.status === "failed") {
      tasks.restoreFailed(failedEnrichmentTask(job), () => retryEnrichmentJob(job.id));
      tasks.setObservedRetry(job.id, () => retryEnrichmentJob(job.id));
    } else tasks.restoreRunning(runningEnrichmentTask(job), () => cancelEnrichmentJob(job.id));
  }
}

function reportSubscriptionFailure(error: unknown) {
  const tasks = useTaskStore();
  tasks.start(
    i18n.global.t("spaces.processing.queue.title"),
    undefined,
    undefined,
    "space-enrichment-observation",
  );
  tasks.fail("space-enrichment-observation", errorMessage(error));
}

function projectMediaProgress(context: SpaceEnrichmentObserverContext, payload: EnrichmentJobUpdate) {
  if (context.activeSpaceId.value === payload.spaceId && payload.jobType === "transcribe_media") {
    if (payload.status === "running") {
      if (!context.analysisIds.value.includes(payload.nodeId))
        context.analysisIds.value = [...context.analysisIds.value, payload.nodeId];
      if (payload.progress?.partial)
        appendSpaceMediaAnalysisPreview({
          previews: context.analysisPreviews,
          nodeId: payload.nodeId,
          progress: payload.progress.progress,
          partial: payload.progress.partial,
        });
    } else {
      context.analysisIds.value = context.analysisIds.value.filter((id) => id !== payload.nodeId);
      clearSpaceMediaAnalysisPreview(context.analysisPreviews, payload.nodeId);
    }
  }
}
