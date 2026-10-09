import { platform } from "@tauri-apps/plugin-os";

import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";
import { invokeCommand } from "../ipc/invoke-command";

export function performApplicationMenuEditing(command: EditingCommand): void | Promise<void> {
  if (platform() !== "macos") return;
  return invokeCommand<boolean>("plugin:desktop|perform_editing", { command }).then(() => undefined);
}
