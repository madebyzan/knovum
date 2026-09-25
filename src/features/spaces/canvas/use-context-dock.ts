import { inject } from "vue";

import { workspaceInspectorKey } from "../../../shared/contracts/workspace-inspector";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { AskInspectorOptions, OpenSourceOptions } from "../inspector/inspector-actions";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { useSpaceStore } from "../session/space-store";
import { useCanvasEditorSession } from "./editor-session";

const DOCK_WIDTH = {
  inspect: 380,
  ask: 420,
  source: 420,
} as const;

export function useContextDock() {
  const store = useSpaceStore();
  const editorSession = useCanvasEditorSession();
  const inspector = inject(workspaceInspectorKey);

  function openInspect() {
    store.setInspectorMode("inspect");
    inspector?.open(DOCK_WIDTH.inspect, "floating");
  }

  function openKnowledgeHealth() {
    editorSession.clearSelection();
    openInspect();
  }

  function openAsk(scope?: ConversationScope, options: AskInspectorOptions = {}) {
    if (scope) store.requestAsk(scope, options);
    else store.setInspectorMode("ask");
    inspector?.open(DOCK_WIDTH.ask, "dock");
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    store.openSource(target, options);
    inspector?.open(DOCK_WIDTH.source, "dock");
  }

  function openWorkflowCapture(nodeIds: string[]) {
    store.openWorkflowCapture(nodeIds);
    inspector?.close();
  }

  function openWorkflowUse(nodeIds: string[]) {
    store.openWorkflowUse(nodeIds);
    inspector?.close();
  }

  function closeDock() {
    if (store.inspectorMode === "source") store.closeSource();
    inspector?.close();
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
