import { watch, type ComputedRef, type Ref } from "vue";

import type { useSpaceStore } from "../../session/space-store";
import type { GroundedMessage } from "../model/grounded-message";
import type { GroundedScope } from "../model/grounded-scope";
import { suggestedKnowledgeScope } from "./assistant-state";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export interface KnowledgeScopeRequestContext {
  store: SpaceStore;
  threadId: Ref<string | undefined>;
  messages: ComputedRef<GroundedMessage[]>;
  scope: Ref<GroundedScope>;
  resetThread: () => void;
  selectedContentIds: ComputedRef<string[]>;
  sectionIds: ComputedRef<string[]>;
  contentNodes: ComputedRef<unknown[]>;
}

export function useSpaceKnowledgeScopeRequests(input: KnowledgeScopeRequestContext) {
  let handledRequest = 0;
  watch(
    () => input.store.askScopeRequest,
    (request) => {
      if (!request || request.nonce === handledRequest) return;
      handledRequest = request.nonce;
      input.store.clearAskScopeRequest();
      const hasConversation = Boolean(input.threadId.value || input.messages.value.length);
      if (request.restart && hasConversation) input.resetThread();
      if (hasConversation) return;
      input.scope.value = scopeAvailable(input, request.scope)
        ? request.scope
        : suggestedKnowledgeScope(input.selectedContentIds.value, input.sectionIds.value);
    },
    { immediate: true },
  );
}

function scopeAvailable(input: KnowledgeScopeRequestContext, scope: GroundedScope) {
  if (scope === "selection") return input.selectedContentIds.value.length > 0;
  if (scope === "section") return input.sectionIds.value.length > 0;
  if (scope === "all-spaces") return true;
  return input.contentNodes.value.length > 0;
}
