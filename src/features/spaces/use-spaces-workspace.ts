import { computed, defineAsyncComponent } from "vue";

import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { useSpaceCommands } from "./commands/use-space-commands";
import { useSpaceStore } from "./session/space-store";

export function useSpacesWorkspace(): WorkspaceDefinition {
  const store = useSpaceStore();
  return {
    view: defineAsyncComponent(() => import("./SpaceView.vue")),
    sidebar: defineAsyncComponent(() => import("./workspace/presentation/SpaceSidebar.vue")),
    rightSidebar: defineAsyncComponent(() => import("./workspace/presentation/SpaceRightSidebar.vue")),
    toolbarLeading: defineAsyncComponent(() => import("./workspace/presentation/SpaceHistoryControls.vue")),
    rightSidebarLayout: { defaultSize: 380, minSize: 300, maxSize: 480 },
    openDefaultRightSidebar: () => {
      store.workflowMode = undefined;
      store.setRightSidebarView("ask");
    },
    rightSidebarAvailable: computed(() => store.workbench.rightSidebar),
    currentScope: computed(() =>
      store.activeSpace
        ? { kind: "space", id: store.activeSpace.id, label: store.activeSpace.title }
        : undefined,
    ),
    toolbar: defineAsyncComponent(() => import("./workspace/presentation/SpaceWorkbenchTabs.vue")),
    useCommands: useSpaceCommands,
  };
}
