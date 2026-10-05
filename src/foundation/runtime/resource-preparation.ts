import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";

import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import type { ResourcePreparationTarget } from "../../shared/contracts/generated/ResourcePreparationTarget";
import { createAppError, decodeAppError } from "../errors/app-error";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import { formatBytes } from "../format/bytes";
import { i18n } from "../i18n/i18n";
import { useTaskStore } from "../tasks/task-store";
import { cancelResourcePreparation, listResourcePreparations, prepareResources } from "./local-resources";
import { invalidateLocalResources } from "./resource-query";

const CANCELLED_CODE = "local_resource_preparation_cancelled";
const OBSERVATION_RETRIES = 3;
const OBSERVATION_INTERVAL_MS = 1_000;

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
    await cancelResourcePreparation(jobId);
    cancelled = true;
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
    await prepareResources({
      target,
      jobId,
      title,
      onProgress: (progress) => {
        updateTaskProgress(tasks, jobId, progress);
        onProgress?.(progress);
      },
    });
    tasks.succeed(jobId);
  } catch (error) {
    if (isCancelled() || decodeAppError(error)?.code === CANCELLED_CODE) {
      tasks.cancelled(jobId);
      throw createAppError(CANCELLED_CODE, "Resource preparation cancelled");
    }
    tasks.fail(jobId, getErrorMessage(error));
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
  async function poll(failures: number): Promise<void> {
    if (!tasks.running.some(({ id }) => id === jobId)) return;
    let snapshot: ResourcePreparationSnapshot | undefined;
    try {
      snapshot = (await listResourcePreparations()).find(({ jobId: candidate }) => candidate === jobId);
      if (!snapshot) throw new Error("Resource preparation snapshot is unavailable");
    } catch (error) {
      if (failures + 1 >= OBSERVATION_RETRIES) {
        tasks.fail(jobId, `Unable to observe resource preparation: ${getErrorMessage(error)}`);
        tasks.setObservedRetry(jobId, () => observeRunning(tasks, jobId));
        await Promise.resolve(
          logError(`Unable to observe resource preparation ${jobId}: ${formatErrorDiagnostic(error)}`),
        ).catch(() => undefined);
        return;
      }
      await new Promise<void>((resolve) => globalThis.setTimeout(resolve, OBSERVATION_INTERVAL_MS));
      return poll(failures + 1);
    }
    updateTaskProgress(tasks, jobId, snapshot.progress);
    if (snapshot.state === "ready") return tasks.succeed(jobId);
    if (snapshot.state === "failed")
      return tasks.fail(jobId, snapshot.error?.message ?? "Resource preparation failed");
    if (snapshot.state === "cancelled") return tasks.cancelled(jobId);
    await new Promise<void>((resolve) => globalThis.setTimeout(resolve, OBSERVATION_INTERVAL_MS));
    return poll(0);
  }
  return poll(0);
}

async function retrySnapshot(
  tasks: ReturnType<typeof useTaskStore>,
  snapshot: ResourcePreparationSnapshot,
): Promise<void> {
  await prepareResources({
    target: snapshot.target,
    jobId: snapshot.jobId,
    title: snapshot.title,
    onProgress: (progress) => updateTaskProgress(tasks, snapshot.jobId, progress),
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
