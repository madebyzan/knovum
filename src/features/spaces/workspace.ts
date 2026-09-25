import { computed, defineAsyncComponent } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { useSpaceCommands } from "./commands/space-command-catalog";
import { useSpaceStore } from "./session/space-store";

export function useSpacesWorkspace(): WorkspaceDefinition {
  const store = useSpaceStore();
  return {
    view: defineAsyncComponent(() => import("./SpaceView.vue")),
    sidebar: defineAsyncComponent(() => import("./presentation/SpaceSidebar.vue")),
    rightSidebar: defineAsyncComponent(() => import("./presentation/SpaceRightSidebar.vue")),
    rightSidebarToolbar: defineAsyncComponent(() => import("./presentation/SpaceRightSidebarToolbar.vue")),
    toolbarLeading: defineAsyncComponent(() => import("./presentation/SpaceHistoryControls.vue")),
    rightSidebarLayout: { defaultSize: 380, minSize: 300, maxSize: 480 },
    openDefaultRightSidebar: () => {
      store.workflowMode = undefined;
      store.setRightSidebarView("ask");
    },
    rightSidebarAvailable: computed(() => store.workspaceMode === "space" && Boolean(store.activeSpaceId)),
    toolbar: defineAsyncComponent(() => import("./presentation/SpaceWorkspaceTabs.vue")),
    commands: useSpaceCommands(),
  };
}
