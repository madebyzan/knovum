import { error as logError } from "@tauri-apps/plugin-log";
import { promiseTimeout } from "@vueuse/core";

import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import type { useTaskStore } from "../tasks/task-store";
import { cancelResourcePreparation, listResourcePreparations } from "./local-resources";
import { resourcePreparationTask, updateTaskProgress } from "./resource-progress";

const OBSERVATION_RETRIES = 3;
const OBSERVATION_INTERVAL_MS = 1_000;

export function restoreRunningPreparation(
  tasks: ReturnType<typeof useTaskStore>,
  snapshot: ResourcePreparationSnapshot,
) {
  const cancel = async () => {
    await cancelResourcePreparation(snapshot.jobId);
    const task = tasks.tasks.find(({ id }) => id === snapshot.jobId);
    if (task?.status !== "attention") return;
    tasks.resumeObservation(snapshot.jobId);
    void observeRunning(tasks, snapshot.jobId);
  };
  const restored = tasks.restoreRunning(resourcePreparationTask(snapshot), cancel);
  if (restored) void observeRunning(tasks, snapshot.jobId);
}

async function observeRunning(tasks: ReturnType<typeof useTaskStore>, jobId: string): Promise<void> {
  let failures = 0;
  /* oxlint-disable no-await-in-loop -- One producer snapshot stream must be polled serially; parallel reads can reorder its terminal state. */
  while (true) {
    const task = tasks.tasks.find(({ id }) => id === jobId);
    if (!task || (task.status !== "running" && task.status !== "attention")) return;
    let snapshot: ResourcePreparationSnapshot | undefined;
    try {
      snapshot = (await listResourcePreparations()).find(({ jobId: candidate }) => candidate === jobId);
      if (!snapshot) throw new Error("Resource preparation snapshot is unavailable");
    } catch (error) {
      failures += 1;
      if (failures >= OBSERVATION_RETRIES) {
        tasks.markObservationUnavailable(
          jobId,
          `Unable to observe resource preparation: ${getErrorMessage(error)}`,
          () => observeRunning(tasks, jobId),
        );
        await Promise.resolve(
          logError(`Unable to observe resource preparation ${jobId}: ${formatErrorDiagnostic(error)}`),
        ).catch(() => undefined);
        return;
      }
      await promiseTimeout(OBSERVATION_INTERVAL_MS);
      continue;
    }
    failures = 0;
    if (applySnapshot(tasks, jobId, snapshot)) return;
    await promiseTimeout(OBSERVATION_INTERVAL_MS);
  }
  /* oxlint-enable no-await-in-loop */
}

function applySnapshot(
  tasks: ReturnType<typeof useTaskStore>,
  jobId: string,
  snapshot: ResourcePreparationSnapshot,
): boolean {
  tasks.resumeObservation(jobId);
  updateTaskProgress(tasks, jobId, snapshot.progress);
  switch (snapshot.state) {
    case "ready":
      tasks.succeed(jobId);
      return true;
    case "failed":
      tasks.fail(jobId, snapshot.error?.message ?? "Resource preparation failed");
      return true;
    case "cancelled":
      tasks.cancelled(jobId);
      return true;
    case "queued":
    case "preparing":
      return false;
  }
}
