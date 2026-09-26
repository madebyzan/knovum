import { warn as logWarning } from "@tauri-apps/plugin-log";
import pLimit from "p-limit";

import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import {
  cancelEnrichmentJob,
  listEnrichmentJobs,
  runEnrichmentJob,
  type SpaceEnrichmentJob,
  type SpaceEnrichmentJobRecord,
} from "./enrichment-jobs";
import { enrichmentJobTitle, failedEnrichmentTask, runningEnrichmentTask } from "./job-tasks";

export interface SpaceEnrichmentSchedulerContext {
  applyNodeDataForSpace: (spaceId: string, id: string, data: SpaceNodePayload) => void;
}

const scheduledJobs = new Set<string>();
const nodeLanes = new Map<string, { limit: ReturnType<typeof pLimit>; users: number }>();

// A committed asset is never rolled back because dispatch failed. Pending intents remain in Rust;
// dispatch errors have their own visible, retryable Task Center entry.
export async function resumePendingEnrichmentJobs(
  context: SpaceEnrichmentSchedulerContext,
  spaceId: string | undefined,
): Promise<void> {
  if (!spaceId) return;
  const targetSpaceId = spaceId;
  const tasks = useTaskStore();
  const failureId = `space-enrichment-dispatch:${spaceId}`;
  try {
    await schedule();
  } catch (error) {
    tasks.restoreFailed(
      {
        id: failureId,
        title: i18n.global.t("spaces.processing.queue.title"),
        detail: String(error),
        startedAt: Date.now(),
      },
      schedule,
    );
    void logWarning(`Space enrichment dispatch failed: ${spaceId} ${String(error)}`);
  }

  async function schedule() {
    const persisted = await listEnrichmentJobs(targetSpaceId);
    const ordered = persisted.toSorted((left, right) => priority(left.jobType) - priority(right.jobType));
    for (const record of ordered) {
      if (scheduledJobs.has(record.id)) continue;
      const job = jobFromRecord(record);
      if (record.status === "failed") {
        tasks.restoreFailed(failedEnrichmentTask(record), () => queueRetry(context, job));
      } else if (record.status === "pending" || record.status === "awaiting_configuration") {
        tasks.restoreRunning(runningEnrichmentTask(record), () => cancelEnrichmentJob(record.id));
        void runForNode(job, () => runJob(context, job)).catch((error: unknown) => {
          tasks.fail(job.id, String(error));
          tasks.setRetry(job.id, () => queueRetry(context, job));
          void logWarning(`Space enrichment job failed: ${job.id} ${String(error)}`);
        });
      }
    }
    tasks.dismiss(failureId);
  }
}

// Serialize only a node's returned projections. Other nodes submit immediately; Rust owns CPU lanes.
// A long Whisper job must not hold a global Renderer batch barrier over new audio analysis.
function runForNode(job: SpaceEnrichmentJob, operation: () => Promise<void>): Promise<void> {
  const key = `${job.spaceId}:${job.nodeId}`;
  const lane = nodeLanes.get(key) ?? { limit: pLimit(1), users: 0 };
  nodeLanes.set(key, lane);
  lane.users += 1;
  scheduledJobs.add(job.id);
  return lane.limit(operation).finally(() => {
    scheduledJobs.delete(job.id);
    lane.users -= 1;
    if (lane.users === 0) nodeLanes.delete(key);
  });
}

function priority(type: SpaceEnrichmentJob["type"]) {
  if (type === "analyze_media") return 0;
  return type === "transcribe_media" ? 1 : 2;
}

async function runJob(context: SpaceEnrichmentSchedulerContext, job: SpaceEnrichmentJob) {
  const tasks = useTaskStore();
  if (await executeJob(context, job)) {
    tasks.succeed(job.id);
    await resumePendingEnrichmentJobs(context, job.spaceId);
  } else {
    tasks.dismiss(job.id);
  }
}

function queueRetry(context: SpaceEnrichmentSchedulerContext, job: SpaceEnrichmentJob) {
  return runForNode(job, async () => {
    useTaskStore().setCancellation(job.id, () => cancelEnrichmentJob(job.id));
    if (await executeJob(context, job)) await resumePendingEnrichmentJobs(context, job.spaceId);
    else useTaskStore().dismiss(job.id);
  });
}

export async function retrySpaceEnrichmentJob(
  context: SpaceEnrichmentSchedulerContext,
  record: SpaceEnrichmentJobRecord,
): Promise<void> {
  const tasks = useTaskStore();
  tasks.restoreFailed(failedEnrichmentTask(record), () => queueRetry(context, jobFromRecord(record)));
  await tasks.retry(record.id);
}

async function executeJob(context: SpaceEnrichmentSchedulerContext, job: SpaceEnrichmentJob) {
  const tasks = useTaskStore();
  const completed = await runEnrichmentJob(job, ({ progress, stage }) => {
    const detail = stage === "analyzing" ? enrichmentJobTitle("analyze_media") : stage;
    tasks.progress(job.id, progress, detail);
  });
  if (!completed) return false;
  context.applyNodeDataForSpace(job.spaceId, completed.nodeId, completed.data);
  return true;
}

function jobFromRecord(record: SpaceEnrichmentJobRecord): SpaceEnrichmentJob {
  return {
    id: record.id,
    spaceId: record.spaceId,
    nodeId: record.nodeId,
    sourceRevision: record.sourceRevision,
    type: record.jobType,
  };
}
