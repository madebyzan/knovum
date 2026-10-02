import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import { canRunAction } from "../../shared/contracts/action-availability";
import type { AppCommand } from "../../shared/contracts/app-command";
import { formatErrorDiagnostic, getErrorMessage } from "../errors/error-format";
import { isApplicationTransitioning } from "../lifecycle/application-transition";

export function runAppCommand(command: AppCommand): void {
  if (isApplicationTransitioning()) return;
  if (command.availability && !canRunAction(command.availability())) return;
  void Promise.resolve()
    .then(() => {
      if (!isApplicationTransitioning()) return command.run();
    })
    .catch((error) => {
      void logError(`Command ${command.id} failed: ${formatErrorDiagnostic(error)}`);
      toast.error(getErrorMessage(error));
    });
}
