import type { InjectionKey, Ref } from "vue";

export interface WorkspaceSidebarPresentation {
  pinned: Readonly<Ref<boolean>>;
  reveal(): void;
  collapse(): void;
}

export const workspaceSidebarKey: InjectionKey<WorkspaceSidebarPresentation> = Symbol("workspace-sidebar");
