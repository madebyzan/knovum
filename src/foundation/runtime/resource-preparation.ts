import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";

import { formatBytes } from "../format/bytes";
import { i18n } from "../i18n/i18n";
import { decodeCommandError } from "../ipc/command-error";
import { useTaskStore } from "../tasks/task-store";
import {
  cancelResourcePreparation,
  listResourcePreparations,
  prepareResources,
  type ResourcePreparationProgress,
  type ResourcePreparationSnapshot,
  type ResourcePreparationTarget,
} from "./local-resources";
import { invalidateLocalResources } from "./resource-query";

const CANCELLED_CODE = "local_resource_preparation_cancelled";

export class ResourcePreparationCancelledError extends Error {}

export interface ResourcePreparationHandle {
  completion: Promise<void>;
  cancel: () => Promise<void>;
}

interface PreparationRun {
  tasks: ReturnType<typeof useTaskStore>;
  target: ResourcePreparationTarget;
  jobId: string;
  title: string;
  isCancelled: () => boolean;
  onProgress?: ((progress: ResourcePreparationProgress) => void) | undefined;
}

export function startResourcePreparation(
  target: ResourcePreparationTarget,
  title: string,
  onProgress?: (progress: ResourcePreparationProgress) => void,
): ResourcePreparationHandle {
  const tasks = useTaskStore();
  const jobId = `runtime-resource:${crypto.randomUUID()}`;
  let cancelled = false;
  const cancel = async () => {
    cancelled = true;
    await cancelResourcePreparation(jobId);
  };
  tasks.restoreRunning({ id: jobId, title, startedAt: Date.now() }, cancel);
  return {
    completion: runPreparation({
      tasks,
      target,
      jobId,
      title,
      isCancelled: () => cancelled,
      onProgress,
    }),
    cancel,
  };
}

export async function restoreResourcePreparationTasks(pinia: Pinia): Promise<void> {
  const tasks = useTaskStore(pinia);
  for (const snapshot of await listResourcePreparations()) {
    if (snapshot.state === "queued" || snapshot.state === "preparing") {
      restoreRunning(tasks, snapshot);
    } else if (snapshot.state === "failed") {
      tasks.restoreFailed(snapshotTask(snapshot), () => retrySnapshot(tasks, snapshot));
    } else {
      tasks.start(snapshot.title, undefined, undefined, snapshot.jobId);
      if (snapshot.state === "ready") tasks.succeed(snapshot.jobId);
      else tasks.cancelled(snapshot.jobId);
    }
  }
}

async function runPreparation(options: PreparationRun) {
  const { tasks, target, jobId, title, isCancelled, onProgress } = options;
  try {
    await prepareResources(target, jobId, title, (progress) => {
      updateTaskProgress(tasks, jobId, progress);
      onProgress?.(progress);
    });
    tasks.succeed(jobId);
  } catch (error) {
    if (isCancelled() || decodeCommandError(error)?.code === CANCELLED_CODE) {
      tasks.cancelled(jobId);
      throw new ResourcePreparationCancelledError("Resource preparation cancelled");
    }
    tasks.fail(jobId, String(error));
    throw error;
  } finally {
    await invalidateLocalResources();
  }
}

function restoreRunning(tasks: ReturnType<typeof useTaskStore>, snapshot: ResourcePreparationSnapshot) {
  const restored = tasks.restoreRunning(snapshotTask(snapshot), () =>
    cancelResourcePreparation(snapshot.jobId),
  );
  if (restored) void observeRunning(tasks, snapshot.jobId);
}

async function observeRunning(tasks: ReturnType<typeof useTaskStore>, jobId: string): Promise<void> {
  async function poll(): Promise<void> {
    if (!tasks.running.some(({ id }) => id === jobId)) return;
    let snapshot: ResourcePreparationSnapshot | undefined;
    try {
      snapshot = (await listResourcePreparations()).find(({ jobId: candidate }) => candidate === jobId);
    } catch (error) {
      await logError(`Unable to observe resource preparation ${jobId}: ${String(error)}`);
      return;
    }
    if (!snapshot) return;
    updateTaskProgress(tasks, jobId, snapshot.progress);
    if (snapshot.state === "ready") return tasks.succeed(jobId);
    if (snapshot.state === "failed")
      return tasks.fail(jobId, snapshot.error?.message ?? "Resource preparation failed");
    if (snapshot.state === "cancelled") return tasks.cancelled(jobId);
    globalThis.setTimeout(() => void poll(), 1_000);
  }
  await poll();
}

async function retrySnapshot(
  tasks: ReturnType<typeof useTaskStore>,
  snapshot: ResourcePreparationSnapshot,
): Promise<void> {
  await prepareResources(snapshot.target, snapshot.jobId, snapshot.title, (progress) => {
    updateTaskProgress(tasks, snapshot.jobId, progress);
  });
  await invalidateLocalResources();
}

function snapshotTask(snapshot: ResourcePreparationSnapshot) {
  return {
    id: snapshot.jobId,
    title: snapshot.title,
    startedAt: snapshot.startedAt,
    ...(snapshot.error === null ? {} : { statusMessage: snapshot.error.message }),
    ...(snapshot.progress.percent === null ? {} : { progress: snapshot.progress.percent }),
  };
}

function updateTaskProgress(
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
  progress: ResourcePreparationProgress,
) {
  if (progress.stage === "queued" || progress.stage === "installing") {
    tasks.indeterminate(taskId);
    return;
  }
  if (progress.stage === "verifying") {
    tasks.indeterminate(taskId, i18n.global.t("tasks.verifyingResource"));
    return;
  }
  if (progress.stage === "ready") {
    tasks.progress(taskId, 100);
    return;
  }
  const statusMessage =
    progress.total && progress.total > 0
      ? `${formatBytes(progress.downloaded)} / ${formatBytes(progress.total)}`
      : undefined;
  if (progress.percent === null) tasks.indeterminate(taskId, statusMessage);
  else tasks.progress(taskId, progress.percent, statusMessage);
}
