import type { InjectionKey } from "vue";

export type WorkspaceInspectorPresentation = "floating" | "dock";

export interface WorkspaceInspectorController {
  open: (preferredSize?: number, presentation?: WorkspaceInspectorPresentation) => void;
  close: () => void;
}

export const workspaceInspectorKey: InjectionKey<WorkspaceInspectorController> =
  Symbol("workspace-inspector");
