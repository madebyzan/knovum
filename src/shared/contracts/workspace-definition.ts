import type { Component, ComputedRef } from "vue";

import type { AppCommand } from "./app-command";

export interface WorkspaceDefinition {
  view: Component;
  sidebar: Component;
  inspector: Component;
  toolbarLeading: Component;
  toolbar: Component;
  inspectorLayout: {
    minSize: number;
    maxSize: number;
  };
  commands: ComputedRef<readonly AppCommand[]>;
}
