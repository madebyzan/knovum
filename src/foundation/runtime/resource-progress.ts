import type { ResourcePreparationProgress } from "../../shared/contracts/generated/ResourcePreparationProgress";
import { formatBytes } from "../format/bytes";
import { i18n } from "../i18n/i18n";
import type { useTaskStore } from "../tasks/task-store";

export function updateTaskProgress(
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
  progress: ResourcePreparationProgress,
) {
  if (progress.stage.startsWith("runtime")) {
    updateRuntimeProgress(tasks, taskId, progress);
    return;
  }
  if (progress.stage === "queued") {
    tasks.indeterminate(
      taskId,
      progress.total
        ? i18n.global.t("tasks.localResourceDownloadSize", { size: formatBytes(progress.total) })
        : undefined,
    );
    return;
  }
  if (progress.stage === "installing") {
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

function updateRuntimeProgress(
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
  progress: ResourcePreparationProgress,
) {
  const component = i18n.global.t("tasks.preparingLocalInference");
  if (progress.stage !== "runtimeDownloading") {
    tasks.indeterminate(taskId, component);
    return;
  }
  const transfer = `${component} ${formatBytes(progress.downloaded)} / ${formatBytes(progress.total!)}`;
  if (progress.percent === null) tasks.indeterminate(taskId, transfer);
  else tasks.progress(taskId, progress.percent, transfer);
}
