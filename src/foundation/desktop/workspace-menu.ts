import { invokeCommand as invoke } from "../ipc/invoke-command";

export function syncWorkspaceMenuState(state: {
  workspaceActive: boolean;
  canClose: boolean;
  canReopen: boolean;
}) {
  return invoke<void>("plugin:desktop|set_workspace_menu_state", state);
}
