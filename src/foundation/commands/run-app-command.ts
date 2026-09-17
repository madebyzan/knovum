import { error as logError } from "@tauri-apps/plugin-log";
import { toast } from "vue-sonner";

import type { AppCommand } from "../../shared/contracts/app-command";

export function runAppCommand(command: AppCommand): void {
  if (command.isEnabled && !command.isEnabled()) return;
  void Promise.resolve()
    .then(() => command.run())
    .catch((error) => {
      void logError(`Command ${command.id} failed: ${String(error)}`);
      toast.error(String(error));
    });
}
