import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import type { ResourcePreparationSnapshot } from "../../shared/contracts/generated/ResourcePreparationSnapshot";
import { formatBytes } from "../format/bytes";
import { i18n } from "../i18n/instance";
import type { useTaskStore } from "../tasks/task-store";

export function formatResourceProgressMessage(progress: ResourcePreparationProgress): string {
  const total = progress.total;
  const hasTotal = total !== null && total > 0;
  switch (progress.stage) {
    case "verifying":
      return i18n.global.t("tasks.resources.verifying");
    case "installing":
      return i18n.global.t("tasks.resources.installing");
    case "queued":
      return hasTotal
        ? i18n.global.t("tasks.resources.downloadSize", { size: formatBytes(total) })
        : i18n.global.t("tasks.resources.preparingInference");
    default: {
      const transfer = hasTotal
        ? `${formatBytes(progress.downloaded)} / ${formatBytes(total)}`
        : formatBytes(progress.downloaded);
      return transfer;
    }
  }
}

export function resolveResourcePreparationPercent(progress: ResourcePreparationProgress): number | null {
  return ["downloading", "ready"].includes(progress.stage) ? progress.percent : null;
}

export function updateTaskProgress(
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
  progress: ResourcePreparationProgress,
) {
  const message = formatResourceProgressMessage(progress);
  const percent = resolveResourcePreparationPercent(progress);
  if (percent === null) tasks.indeterminate(taskId, message);
  else tasks.progress(taskId, percent, message);
}

export function buildResourcePreparationTask(snapshot: ResourcePreparationSnapshot) {
  const percent = resolveResourcePreparationPercent(snapshot.progress);
  return {
    id: snapshot.jobId,
    title: snapshot.title,
    startedAt: snapshot.startedAt,
    ...(snapshot.error === null ? {} : { statusMessage: snapshot.error.message }),
    ...(percent === null ? {} : { progress: percent }),
  };
}
