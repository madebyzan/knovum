import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import { canRunAction } from "../../shared/contracts/action-availability";
import type { AppCommand } from "../../shared/contracts/app-command";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import { isApplicationRestarting } from "../lifecycle/window-close-coordinator";

export function runAppCommand(command: AppCommand): void {
  if (isApplicationRestarting()) return;
  if (command.availability && !canRunAction(command.availability())) return;
  void Promise.resolve()
    .then(() => (isApplicationRestarting() ? undefined : command.run()))
    .catch((error) => {
      void logError(`Command ${command.id} failed: ${formatErrorDiagnostic(error)}`);
      toast.error(getErrorMessage(error));
    });
}
