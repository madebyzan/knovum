import type { InjectionKey } from "vue";

export interface WorkspaceSidebarPresentation {
  reveal(): void;
}

export const workspaceSidebarKey: InjectionKey<WorkspaceSidebarPresentation> = Symbol("workspace-sidebar");
