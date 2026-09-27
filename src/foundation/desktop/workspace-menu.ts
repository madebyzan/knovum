import { invokeCommand } from "../ipc/invoke-command";

export function syncWorkspaceMenuState(state: {
  workspaceActive: boolean;
  canClose: boolean;
  canReopen: boolean;
}) {
  return invokeCommand<void>("plugin:desktop|set_workspace_menu_state", state);
}
