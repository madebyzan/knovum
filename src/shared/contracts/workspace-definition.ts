import type { Component, ComputedRef } from "vue";

import type { AppCommand } from "./app-command";

export interface WorkspaceDefinition {
  view: Component;
  sidebar: Component;
  rightSidebar: Component;
  rightSidebarToolbar: Component;
  toolbarLeading: Component;
  toolbar: Component;
  rightSidebarLayout: {
    defaultSize: number;
    minSize: number;
    maxSize: number;
  };
  openDefaultRightSidebar: () => void;
  rightSidebarAvailable: ComputedRef<boolean>;
  commands: ComputedRef<readonly AppCommand[]>;
}
