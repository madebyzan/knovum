import { invokeCommand } from "../ipc/invoke-command";

export function cancelAiJob(jobId: string): Promise<void> {
  return invokeCommand<void>("plugin:ai|cancel_job", { jobId });
}
