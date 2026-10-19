import { platform } from "@tauri-apps/plugin-os";

import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";
import { invokeCommand } from "../ipc/invoke-command";

/**
 * The chords platform text editing owns. A focused editable target always keeps them, so no
 * application command may claim them while the user is typing; macOS routes them through the
 * Application Menu instead, because a WebView never receives them on its own.
 */
export const TEXT_EDITING_CHORDS: readonly string[] = [
  "$mod+Z",
  "$mod+Shift+Z",
  "$mod+Y",
  "$mod+X",
  "$mod+C",
  "$mod+V",
  "$mod+A",
];

export function performApplicationMenuEditing(command: EditingCommand): void | Promise<void> {
  if (platform() !== "macos") return;
  return invokeCommand<boolean>("plugin:desktop|perform_editing", { command }).then(() => undefined);
}
