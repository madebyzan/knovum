import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { RunControlReceipt } from "../contracts/generated/RunControlReceipt";

type RunControlCommand = "cancel" | "pause";

const ipcCommands: Record<RunControlCommand, string> = {
  cancel: "plugin:spaces|cancel_run",
  pause: "plugin:spaces|pause_run",
};

export async function requestRunControl(command: RunControlCommand, runId: string): Promise<void> {
  const receipt = await invokeCommand<RunControlReceipt>(ipcCommands[command], { runId });
  for (const warning of receipt.interruptionErrors) toast.warning(getErrorMessage(warning));
}
