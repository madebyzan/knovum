import type { SpacePlacement } from "../canvas/navigation-requests";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import {
  classifyReadableSource,
  type SourceReaderPane,
  type SourceReaderRequest,
} from "../reading/source-reader-target";
import type { SpaceState } from "../session/session-state";

const MAX_READING_HISTORY = 20;

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
  openDocumentEditor: (nodeId: string) => Promise<void>,
) {
  function requestAsk(scope: ConversationScope, options: AskSidebarOptions = {}) {
    if (!state.workbench.value.rightSidebar) throw new Error("Canvas context is unavailable");
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

  function resolvePane(nodeId: string, requested?: SourceReaderPane): SourceReaderPane | undefined {
    const node = state.nodes.value.find((candidate) => candidate.id === nodeId && !candidate.hidden);
    if (!node) return undefined;
    const readable = Boolean(classifyReadableSource(node.data));
    if (requested === "info") return "info";
    return readable ? "read" : "info";
  }

  function recordHistory(nodeId: string, pane: SourceReaderPane) {
    const head = state.readingHistory.value.slice(0, state.readingHistoryIndex.value + 1);
    const last = head.at(-1);
    if (last?.nodeId === nodeId) {
      head[head.length - 1] = { nodeId, pane };
    } else {
      head.push({ nodeId, pane });
    }
    const trimmed = head.slice(-MAX_READING_HISTORY);
    state.readingHistory.value = trimmed;
    state.readingHistoryIndex.value = trimmed.length - 1;
  }

  function nextReadingNonce() {
    const previous = state.readingTarget.value?.nonce ?? 0;
    return Math.max(Date.now(), previous + 1);
  }

  function openSourceReader(request: SourceReaderRequest) {
    const pane = resolvePane(request.nodeId, request.pane);
    if (!pane) return;
    recordHistory(request.nodeId, pane);
    state.readingTarget.value = {
      nodeId: request.nodeId,
      pane,
      locator: request.locator,
      quote: request.quote ?? request.locator?.quote?.exact,
      nonce: nextReadingNonce(),
    };
  }

  function setReadingPane(pane: SourceReaderPane) {
    const current = state.readingTarget.value;
    if (!current) return;
    const resolved = resolvePane(current.nodeId, pane);
    if (!resolved || resolved === current.pane) return;
    recordHistory(current.nodeId, resolved);
    state.readingTarget.value = { ...current, pane: resolved };
  }

  function stepReadingHistory(delta: -1 | 1) {
    const nextIndex = state.readingHistoryIndex.value + delta;
    const entry = state.readingHistory.value[nextIndex];
    if (!entry) return;
    const pane = resolvePane(entry.nodeId, entry.pane);
    if (!pane) return;
    state.readingHistoryIndex.value = nextIndex;
    state.readingTarget.value = { nodeId: entry.nodeId, pane, nonce: Date.now() };
  }

  function closeSourceReader() {
    state.readingTarget.value = undefined;
    state.readingHistory.value = [];
    state.readingHistoryIndex.value = -1;
  }

  function openSource(target: KnowledgePeekTarget, options: OpenSourceOptions = {}) {
    if (!["canvas", "document"].includes(state.surface.value.kind)) {
      throw new Error("Source context is unavailable");
    }
    openSourceReader({
      nodeId: target.nodeId,
      locator: target.locator,
      quote: target.text ?? target.locator.quote?.exact,
      pane: "read",
    });
    if (state.surface.value.kind === "canvas" && options.reveal !== false) {
      placement.requestReveal([target.nodeId], "ensure-visible");
    }
  }

  function openKnowledgeHealth() {
    state.knowledgeHealthOpen.value = true;
  }

  async function openEditorAtSource(target: KnowledgePeekTarget) {
    const node = state.nodes.value.find((candidate) => candidate.id === target.nodeId);
    if (!node || !["note", "markdown"].includes(node.data.kind)) {
      throw new Error("Source is not editable in Knovum");
    }
    await openDocumentEditor(node.id);
    state.editorLocateRequest.value = { nodeId: node.id, locator: target.locator, nonce: Date.now() };
  }

  return {
    requestAsk,
    clearAskScopeRequest,
    openSource,
    openSourceReader,
    setReadingPane,
    stepReadingHistory,
    closeSourceReader,
    openKnowledgeHealth,
    openEditorAtSource,
  };
}
