import { defineAsyncComponent } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { useSpaceCommands } from "./commands/space-command-catalog";

export function useSpacesWorkspace(): WorkspaceDefinition {
  return {
    view: defineAsyncComponent(() => import("./SpaceView.vue")),
    sidebar: defineAsyncComponent(() => import("./presentation/SpaceSidebar.vue")),
    inspector: defineAsyncComponent(() => import("./presentation/SpaceInspector.vue")),
    toolbarLeading: defineAsyncComponent(() => import("./presentation/SpaceHistoryControls.vue")),
    inspectorLayout: { minSize: 300, maxSize: 480 },
    toolbar: defineAsyncComponent(() => import("./presentation/SpaceWorkspaceTabs.vue")),
    commands: useSpaceCommands(),
  };
}
