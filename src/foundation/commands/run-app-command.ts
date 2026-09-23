import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import type { AppCommand } from "../../shared/contracts/app-command";
import { canRunAction } from "../../shared/contracts/action-availability";

export function runAppCommand(command: AppCommand): void {
  if (command.availability && !canRunAction(command.availability())) return;
  void Promise.resolve()
    .then(() => command.run())
    .catch((error) => {
      void logError(`Command ${command.id} failed: ${String(error)}`);
      toast.error(String(error));
    });
}
