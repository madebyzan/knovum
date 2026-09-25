import type { SpacePlacement } from "../canvas/navigation-requests";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import type { SpaceState } from "../session/session-state";
import type { RightSidebarPrimaryView } from "../session/workbench-state";

export interface AskSidebarOptions {
  restart?: boolean;
  nodeIds?: string[];
}

export interface OpenSourceOptions {
  reveal?: boolean;
}

export function createRightSidebarActions(
  state: SpaceState,
  placement: SpacePlacement,
  requestMediaSeek: (nodeId: string, timeMs: number) => void,
) {
  function setRightSidebarView(mode: RightSidebarPrimaryView) {
    state.sourcePeek.value = undefined;
    state.rightSidebarView.value = mode;
  }

  function requestAsk(scope: ConversationScope, options: AskSidebarOptions = {}) {
    state.sourcePeek.value = undefined;
    state.rightSidebarView.value = "ask";
    state.askScopeRequest.value = {
      scope,
      restart: options.restart ?? false,
      nodeIds: options.nodeIds ?? [],
      nonce: ++state.askScopeNonce.value,
    };
  }

  function clearAskScopeRequest() {
    state.askScopeRequest.value = undefined;
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    const currentView = state.rightSidebarView.value;
    if (currentView !== "source") state.sourceReturnView.value = currentView;
    state.sourcePeek.value = target;
    state.rightSidebarView.value = "source";
    if (options.reveal !== false) placement.requestReveal([target.nodeId], "ensure-visible");
    if (target.locator.location.kind === "media") {
      requestMediaSeek(target.nodeId, target.locator.location.startMs);
    }
  }

  function closeSource() {
    state.sourcePeek.value = undefined;
    state.rightSidebarView.value = state.sourceReturnView.value;
  }

  function openEditorAtSource(target: KnowledgePeekTarget) {
    const node = state.nodes.value.find((candidate) => candidate.id === target.nodeId);
    if (!node || !["note", "markdown"].includes(node.data.kind)) {
      throw new Error("Source is not editable in Knovum");
    }
    state.focusedEditorId.value = node.id;
    state.editorLocateRequest.value = { nodeId: node.id, locator: target.locator, nonce: Date.now() };
  }

  return {
    setRightSidebarView,
    requestAsk,
    clearAskScopeRequest,
    openSource,
    closeSource,
    openEditorAtSource,
  };
}
