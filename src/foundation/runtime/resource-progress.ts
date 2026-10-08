import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import { formatBytes } from "../format/bytes";
import { i18n } from "../i18n/i18n";
import type { useTaskStore } from "../tasks/task-store";

export function resourceProgressMessage(progress: ResourcePreparationProgress): string {
  const total = progress.total;
  const hasTotal = total !== null && total > 0;
  switch (progress.stage) {
    case "verifying":
    case "runtimeVerifying":
      return i18n.global.t("tasks.verifyingResource");
    case "installing":
    case "runtimeInstalling":
      return i18n.global.t("tasks.installingResource");
    case "queued":
      return hasTotal
        ? i18n.global.t("tasks.localResourceDownloadSize", { size: formatBytes(total) })
        : i18n.global.t("tasks.preparingLocalInference");
    default: {
      const transfer = hasTotal
        ? `${formatBytes(progress.downloaded)} / ${formatBytes(total)}`
        : formatBytes(progress.downloaded);
      return progress.stage === "runtimeDownloading"
        ? `${i18n.global.t("tasks.preparingLocalInference")} · ${transfer}`
        : transfer;
    }
  }
}

export function updateTaskProgress(
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
  progress: ResourcePreparationProgress,
) {
  const message = resourceProgressMessage(progress);
  const transferable = ["downloading", "runtimeDownloading", "ready"].includes(progress.stage);
  if (!transferable || progress.percent === null) tasks.indeterminate(taskId, message);
  else tasks.progress(taskId, progress.percent, message);
}

export function resourcePreparationTask(snapshot: ResourcePreparationSnapshot) {
  return {
    id: snapshot.jobId,
    title: snapshot.title,
    startedAt: snapshot.startedAt,
    ...(snapshot.error === null ? {} : { statusMessage: snapshot.error.message }),
    ...(snapshot.progress.percent === null ? {} : { progress: snapshot.progress.percent }),
  };
}
