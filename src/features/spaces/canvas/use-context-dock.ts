import { inject } from "vue";

import { workspaceRightSidebarKey } from "../../../shared/contracts/workspace-right-sidebar";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import type { AskSidebarOptions, OpenSourceOptions } from "../right-sidebar/right-sidebar-actions";
import { useSpaceStore } from "../session/space-store";
import { getCanvasEditorSession } from "./editor-session";

export function useContextDock() {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const rightSidebar = inject(workspaceRightSidebarKey);

  function openInspect() {
    store.workflowMode = undefined;
    store.setRightSidebarView("inspect");
    rightSidebar?.open();
  }

  function openKnowledgeHealth() {
    editorSession.clearSelection();
    openInspect();
  }

  function openAsk(scope?: ConversationScope, options: AskSidebarOptions = {}) {
    store.workflowMode = undefined;
    if (scope) store.requestAsk(scope, options);
    else store.setRightSidebarView("ask");
    rightSidebar?.open();
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    store.workflowMode = undefined;
    store.openSource(target, options);
    rightSidebar?.open();
  }

  function openDocumentOutline() {
    store.workflowMode = undefined;
    store.setRightSidebarView("document-outline");
    rightSidebar?.open();
  }

  function openWorkflowCapture(nodeIds: string[]) {
    store.openWorkflowCapture(nodeIds);
    rightSidebar?.close();
  }

  function openWorkflowUse(nodeIds: string[]) {
    store.openWorkflowUse(nodeIds);
    rightSidebar?.close();
  }

  function closeDock() {
    if (store.rightSidebarView === "source") store.closeSource();
    rightSidebar?.close();
  }

  return {
    openInspect,
    openKnowledgeHealth,
    openAsk,
    openSource,
    openDocumentOutline,
    openWorkflowCapture,
    openWorkflowUse,
    closeDock,
  };
}
