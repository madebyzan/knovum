import { Channel } from "@tauri-apps/api/core";

import type { PluginOperationStage } from "../../shared/contracts/generated/PluginOperationStage";
import { getErrorMessage } from "../errors/error-format";
import { i18n } from "../i18n/i18n";
import { invokeCommand } from "../ipc/invoke-command";
import { useTaskStore } from "../tasks/task-store";

const stageMessageKeys: Record<PluginOperationStage, string> = {
  reading: "extensions.operationStages.reading",
  validating: "extensions.operationStages.validating",
  committing: "extensions.operationStages.committing",
};

export async function runPluginOperation<T>(
  command: string,
  titleKey: string,
  payload: Record<string, unknown>,
): Promise<T> {
  const tasks = useTaskStore();
  const jobId = crypto.randomUUID();
  const cancel = () => invokeCommand<void>("plugin:extensions|cancel_plugin_operation", { jobId });
  tasks.restoreRunning({ id: jobId, title: i18n.global.t(titleKey), startedAt: Date.now() }, cancel);
  const onProgress = new Channel<PluginOperationStage>((stage) => {
    tasks.indeterminate(jobId, i18n.global.t(stageMessageKeys[stage]));
    if (stage === "committing") tasks.setCancellation(jobId);
  });
  try {
    const response = await invokeCommand<T>(`plugin:extensions|${command}`, {
      ...payload,
      jobId,
      onProgress,
    });
    tasks.succeed(jobId);
    return response;
  } catch (error) {
    const message = getErrorMessage(error);
    if (message === "plugin-operation-cancelled") {
      tasks.cancelled(jobId);
      throw new Error(i18n.global.t("extensions.operationCancelled"), { cause: error });
    }
    tasks.fail(jobId, message);
    throw error;
  }
}
