import type { Component, ComputedRef } from "vue";

import type { AppCommand } from "./app-command";
import type { TaskReference } from "./task-reference";

export interface WorkspaceDefinition {
  view: Component;
  sidebar: Component;
  rightSidebar: Component;
  toolbarLeading: Component;
  toolbar: Component;
  rightSidebarLayout: {
    defaultSize: number;
    minSize: number;
    maxSize: number;
  };
  openDefaultRightSidebar: () => void;
  rightSidebarAvailable: ComputedRef<boolean>;
  currentScope?: ComputedRef<TaskReference | undefined>;
  commands: ComputedRef<readonly AppCommand[]>;
}
