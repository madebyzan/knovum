import { inject } from "vue";

import {
  workspaceRightSidebarKey,
  type WorkspaceRightSidebarController,
} from "../../../shared/contracts/workspace-right-sidebar";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { useSpaceStore } from "../session/space-store";
import type { AskSidebarOptions, OpenSourceOptions } from "../workspace/right-sidebar-actions";
import { getCanvasEditorSession } from "./editor-session";

export function useContextDock() {
  const rightSidebar = inject(workspaceRightSidebarKey);
  if (!rightSidebar) throw new Error("Context dock requires the workspace right sidebar");
  return createContextDock(rightSidebar);
}

export function createContextDock(rightSidebar: WorkspaceRightSidebarController) {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();

  function openInspect() {
    store.workflowMode = undefined;
    store.setRightSidebarView("inspect");
    rightSidebar.open();
  }

  function openKnowledgeHealth() {
    editorSession.clearSelection();
    openInspect();
  }

  function openAsk(scope?: ConversationScope, options: AskSidebarOptions = {}) {
    store.workflowMode = undefined;
    if (scope) store.requestAsk(scope, options);
    else store.setRightSidebarView("ask");
    rightSidebar.open();
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    store.workflowMode = undefined;
    store.openSource(target, options);
    if (store.workbench.rightSidebar) rightSidebar.open();
  }

  function openWorkflowCapture(nodeIds: string[]) {
    store.openWorkflowCapture(nodeIds);
    rightSidebar.close();
  }

  function openWorkflowUse(nodeIds: string[]) {
    store.openWorkflowUse(nodeIds);
    rightSidebar.close();
  }

  function closeDock() {
    if (store.rightSidebarView === "source") store.closeSource();
    rightSidebar.close();
  }

  return {
    openInspect,
    openKnowledgeHealth,
    openAsk,
    openSource,
    openWorkflowCapture,
    openWorkflowUse,
    closeDock,
  };
}
