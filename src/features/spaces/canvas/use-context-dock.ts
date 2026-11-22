import { inject } from "vue";

import {
  workspaceRightSidebarKey,
  type WorkspaceRightSidebarController,
} from "../../../shared/contracts/workspace-right-sidebar";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { useSpaceStore } from "../session/space-store";
import type { AskSidebarOptions, OpenSourceOptions } from "../workspace/right-sidebar-actions";

export function useContextDock() {
  const rightSidebar = inject(workspaceRightSidebarKey);
  if (!rightSidebar) throw new Error("Context dock requires the workspace right sidebar");
  return createContextDock(rightSidebar);
}

export function createContextDock(rightSidebar: WorkspaceRightSidebarController) {
  const store = useSpaceStore();

  function openNodeInfo(nodeId: string) {
    store.openSourceReader({ nodeId, pane: "info" });
  }

  function openKnowledgeHealth() {
    store.openKnowledgeHealth();
  }

  function openAsk(scope?: ConversationScope, options: AskSidebarOptions = {}) {
    if (scope) store.requestAsk(scope, options);
    rightSidebar.open();
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    store.openSource(target, options);
  }

  function closeDock() {
    rightSidebar.close();
  }

  return {
    openNodeInfo,
    openKnowledgeHealth,
    openAsk,
    openSource,
    closeDock,
  };
}
