import type { SpacePlacement } from "../canvas/navigation-requests";
import type { GroundedScope } from "../knowledge/model/grounded-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import type { SpaceState } from "../session/session-state";

export interface AskInspectorOptions {
  restart?: boolean;
}

export interface OpenSourceOptions {
  reveal?: boolean;
}

export function createInspectorActions(
  state: SpaceState,
  placement: SpacePlacement,
  requestMediaSeek: (nodeId: string, timeMs: number) => void,
) {
  function setInspectorMode(mode: "inspect" | "ask") {
    state.sourcePeek.value = undefined;
    state.inspectorMode.value = mode;
  }

  function requestAsk(scope: GroundedScope, options: AskInspectorOptions = {}) {
    state.sourcePeek.value = undefined;
    state.inspectorMode.value = "ask";
    state.askScopeRequest.value = {
      scope,
      restart: options.restart ?? false,
      nonce: ++state.askScopeNonce.value,
    };
  }

  function clearAskScopeRequest() {
    state.askScopeRequest.value = undefined;
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    if (state.inspectorMode.value !== "source") {
      state.sourceReturnMode.value = state.inspectorMode.value === "ask" ? "ask" : "inspect";
    }
    state.sourcePeek.value = target;
    state.inspectorMode.value = "source";
    if (options.reveal !== false) placement.requestReveal([target.nodeId], "ensure-visible");
    if (target.locator.location.kind === "media") {
      requestMediaSeek(target.nodeId, target.locator.location.startMs);
    }
  }

  function closeSource() {
    state.sourcePeek.value = undefined;
    state.inspectorMode.value = state.sourceReturnMode.value;
  }

  function openEditorAtSource(target: KnowledgePeekTarget) {
    const node = state.nodes.value.find((candidate) => candidate.id === target.nodeId);
    if (!node || !["note", "markdown"].includes(node.data.kind)) {
      throw new Error("Source is not editable in Knovum");
    }
    state.focusedEditorId.value = node.id;
    state.editorLocateRequest.value = { nodeId: node.id, locator: target.locator, nonce: Date.now() };
  }

  return { setInspectorMode, requestAsk, clearAskScopeRequest, openSource, closeSource, openEditorAtSource };
}
