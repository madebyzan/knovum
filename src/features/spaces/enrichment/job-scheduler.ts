import { warn as logWarning } from "@tauri-apps/plugin-log";
import { groupBy } from "es-toolkit";

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

export interface SpaceEnrichmentSchedulerContext {
  applyNodeDataForSpace: (spaceId: string, id: string, data: SpaceNodePayload) => void;
}

const pendingJobs = new Map<string, SpaceEnrichmentJob>();
const scheduledJobs = new Set<string>();
let draining = false;

export async function resumePendingEnrichmentJobs(
  context: SpaceEnrichmentSchedulerContext,
  spaceId: string | undefined,
): Promise<void> {
  if (!spaceId) return;
  const tasks = useTaskStore();
  const persisted = await listEnrichmentJobs(spaceId);
  for (const record of persisted) {
    if (scheduledJobs.has(record.id)) continue;
    const job = jobFromRecord(record);
    if (record.status === "failed") {
      tasks.restoreFailed(failedTask(record), () => retryJob(context, job));
    } else if (record.status === "pending" || record.status === "awaiting_configuration") {
      tasks.restoreRunning(runningTask(record), () => cancelEnrichmentJob(record.id));
      pendingJobs.set(job.id, job);
      scheduledJobs.add(job.id);
    }
  }
  void drain(context);
}

async function drain(context: SpaceEnrichmentSchedulerContext): Promise<void> {
  if (draining) return;
  draining = true;
  try {
    const jobs = Array.from(pendingJobs.values()).sort(
      (left, right) => executionPriority(left) - executionPriority(right),
    );
    pendingJobs.clear();
    const jobsByNode = groupBy(jobs, (job) => job.nodeId);
    await Promise.all(
      Object.values(jobsByNode).map(async (nodeJobs) => {
        for (const job of nodeJobs) {
          // oxlint-disable-next-line eslint/no-await-in-loop -- A node's knowledge payload has one revision stream.
          await runJob(context, job);
        }
      }),
    );
  } finally {
    draining = false;
    if (pendingJobs.size) void drain(context);
  }
}

function executionPriority(job: SpaceEnrichmentJob) {
  return job.type === "transcribe_media" ? 0 : 1;
}

async function runJob(context: SpaceEnrichmentSchedulerContext, job: SpaceEnrichmentJob) {
  const tasks = useTaskStore();
  try {
    if (await executeJob(context, job)) {
      tasks.succeed(job.id);
      await resumePendingEnrichmentJobs(context, job.spaceId);
    } else {
      tasks.dismiss(job.id);
    }
  } catch (error) {
    const message = String(error);
    tasks.fail(job.id, message);
    tasks.setRetry(job.id, () => retryJob(context, job));
    await logWarning(`Optional Space enrichment job failed: ${job.id} ${message}`);
  } finally {
    scheduledJobs.delete(job.id);
  }
}

async function retryJob(context: SpaceEnrichmentSchedulerContext, job: SpaceEnrichmentJob) {
  const tasks = useTaskStore();
  tasks.setCancellation(job.id, () => cancelEnrichmentJob(job.id));
  if (await executeJob(context, job)) await resumePendingEnrichmentJobs(context, job.spaceId);
  else tasks.dismiss(job.id);
}

async function executeJob(context: SpaceEnrichmentSchedulerContext, job: SpaceEnrichmentJob) {
  const tasks = useTaskStore();
  const completed = await runEnrichmentJob(job, ({ progress, stage }) => {
    tasks.progress(job.id, progress, stage);
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

function jobTitle(jobType: SpaceEnrichmentJobRecord["jobType"]) {
  return i18n.global.t(`spaces.processing.enrichmentJobType.${jobType}`);
}

function runningTask(record: SpaceEnrichmentJobRecord) {
  return { id: record.id, title: jobTitle(record.jobType), startedAt: record.createdAt };
}

function failedTask(record: SpaceEnrichmentJobRecord) {
  return {
    id: record.id,
    title: jobTitle(record.jobType),
    detail: record.error ?? i18n.global.t("spaces.processing.processingFailed"),
    startedAt: record.createdAt,
  };
}
