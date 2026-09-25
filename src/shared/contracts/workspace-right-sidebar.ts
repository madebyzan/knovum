import type { InjectionKey } from "vue";

export interface WorkspaceRightSidebarController {
  open: () => void;
  close: () => void;
}

export const workspaceRightSidebarKey: InjectionKey<WorkspaceRightSidebarController> =
  Symbol("workspace-right-sidebar");
