import { defineAsyncComponent } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { useSpaceCommands } from "./commands/space-command-catalog";

export function useSpacesWorkspace(): WorkspaceDefinition {
  return {
    view: defineAsyncComponent(() => import("./SpaceView.vue")),
    sidebar: defineAsyncComponent(() => import("./presentation/workspace/SpaceSidebar.vue")),
    inspector: defineAsyncComponent(() => import("./presentation/inspector/SpaceInspector.vue")),
    toolbarLeading: defineAsyncComponent(() => import("./presentation/workspace/SpaceHistoryControls.vue")),
    inspectorLayout: { minSize: 300, maxSize: 480 },
    toolbar: defineAsyncComponent(() => import("./presentation/workspace/SpaceWorkspaceTabs.vue")),
    commands: useSpaceCommands(),
  };
}
