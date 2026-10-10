import { error as logError } from "@tauri-apps/plugin-log";
import type { Pinia } from "pinia";

import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import type { ResourcePreparationTarget } from "../../shared/contracts/generated/ResourcePreparationTarget";
import { createAppError, decodeAppError } from "../errors/app-error";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import { useTaskStore } from "../tasks/task-store";
import { cancelResourcePreparation, listResourcePreparations, prepareResources } from "./local-resources";
import { restoreRunningPreparation } from "./resource-preparation-observation";
import { buildResourcePreparationTask, updateTaskProgress } from "./resource-progress";
import { invalidateLocalResources } from "./use-resource-queries";

const CANCELLED_CODE = "local_resource_preparation_cancelled";

export interface ResourcePreparationHandle {
  completion: Promise<void>;
  cancel: () => Promise<void>;
}

interface PreparationRun {
  tasks: ReturnType<typeof useTaskStore>;
  target: ResourcePreparationTarget;
  jobId: string;
  title: string;
  onProgress?: ((progress: ResourcePreparationProgress) => void) | undefined;
}

export function startResourcePreparation(
  target: ResourcePreparationTarget,
  title: string,
  onProgress?: (progress: ResourcePreparationProgress) => void,
): ResourcePreparationHandle {
  const tasks = useTaskStore();
  const jobId = `runtime-resource:${crypto.randomUUID()}`;
  const cancel = () => cancelResourcePreparation(jobId);
  tasks.restoreRunning({ id: jobId, title, startedAt: Date.now() }, cancel);
  return {
    completion: runPreparation({
      tasks,
      target,
      jobId,
      title,
      onProgress,
    }),
    cancel,
  };
}

export async function restoreResourcePreparationTasks(pinia: Pinia): Promise<void> {
  const tasks = useTaskStore(pinia);
  for (const snapshot of await listResourcePreparations()) {
    if (snapshot.state === "queued" || snapshot.state === "preparing") {
      restoreRunningPreparation(tasks, snapshot);
    } else if (snapshot.state === "failed") {
      tasks.restoreFailed(buildResourcePreparationTask(snapshot), () => retrySnapshot(tasks, snapshot), {
        settlement: "external",
      });
    } else {
      tasks.start(snapshot.title, undefined, undefined, snapshot.jobId);
      if (snapshot.state === "ready") tasks.succeed(snapshot.jobId);
      else tasks.cancelled(snapshot.jobId);
    }
  }
}

async function runPreparation(options: PreparationRun) {
  const { tasks, target, jobId, title, onProgress } = options;
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
    if (decodeAppError(error)?.code === CANCELLED_CODE) {
      tasks.cancelled(jobId);
      throw createAppError(CANCELLED_CODE, "Resource preparation cancelled");
    }
    tasks.fail(jobId, getErrorMessage(error));
    throw error;
  } finally {
    await invalidateResourceProjection(jobId);
  }
}

async function retrySnapshot(
  tasks: ReturnType<typeof useTaskStore>,
  snapshot: ResourcePreparationSnapshot,
): Promise<void> {
  tasks.setCancellation(snapshot.jobId, () => cancelResourcePreparation(snapshot.jobId));
  await runPreparation({
    tasks,
    target: snapshot.target,
    jobId: snapshot.jobId,
    title: snapshot.title,
  });
}

async function invalidateResourceProjection(jobId: string): Promise<void> {
  try {
    await invalidateLocalResources();
  } catch (error) {
    await Promise.resolve(
      logError(`Unable to refresh local resources after ${jobId}: ${formatErrorDiagnostic(error)}`),
    ).catch(() => undefined);
  }
}
