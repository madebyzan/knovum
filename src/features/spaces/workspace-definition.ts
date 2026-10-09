import { computed, defineAsyncComponent } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { useSpaceCommands } from "./commands/use-space-commands";
import { useSpaceStore } from "./session/space-store";

export function useSpacesWorkspace(): WorkspaceDefinition {
  const store = useSpaceStore();
  return {
    view: defineAsyncComponent(() => import("./SpaceView.vue")),
    sidebar: defineAsyncComponent(() => import("./presentation/workspace/SpaceSidebar.vue")),
    rightSidebar: defineAsyncComponent(() => import("./presentation/workspace/SpaceRightSidebar.vue")),
    toolbarLeading: defineAsyncComponent(() => import("./presentation/workspace/SpaceHistoryControls.vue")),
    rightSidebarLayout: { defaultSize: 380, minSize: 300, maxSize: 480 },
    openDefaultRightSidebar: () => {
      store.workflowMode = undefined;
      store.setRightSidebarView("ask");
    },
    rightSidebarAvailable: computed(() => store.workspaceMode === "space" && Boolean(store.activeSpaceId)),
    currentScope: computed(() =>
      store.activeSpace
        ? { kind: "space", id: store.activeSpace.id, label: store.activeSpace.title }
        : undefined,
    ),
    toolbar: defineAsyncComponent(() => import("./presentation/workspace/SpaceWorkspaceTabs.vue")),
    useCommands: useSpaceCommands,
  };
}
