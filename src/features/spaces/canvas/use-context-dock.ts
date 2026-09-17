import { inject } from "vue";

import { workspaceInspectorKey } from "../../../shared/contracts/workspace-inspector";
import type { AskInspectorOptions, OpenSourceOptions } from "../inspector/inspector-actions";
import type { GroundedScope } from "../knowledge/model/grounded-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import { useSpaceStore } from "../session/space-store";
import { useCanvasEditorSession } from "./editor-session";

const DOCK_WIDTH = {
  inspect: 300,
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

  function openAsk(scope?: GroundedScope, options: AskInspectorOptions = {}) {
    if (scope) store.requestAsk(scope, options);
    else store.setInspectorMode("ask");
    inspector?.open(DOCK_WIDTH.ask, "dock");
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    store.openSource(target, options);
    inspector?.open(DOCK_WIDTH.source, "dock");
  }

  function closeDock() {
    if (store.inspectorMode === "source") store.closeSource();
    inspector?.close();
  }

  return { openInspect, openKnowledgeHealth, openAsk, openSource, closeDock };
}
