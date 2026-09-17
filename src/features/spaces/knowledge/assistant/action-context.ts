import type { QueryClient } from "@tanstack/vue-query";
import type { ComputedRef, Ref } from "vue";

import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedScope } from "../model/grounded-scope";
import type { KnowledgeSearchResponse } from "../model/knowledge-search-response";
import type { GroundedAnswerStage } from "./grounded-assistant";
import type { PendingAsk } from "./use-assistant-session";

export interface KnowledgeAssistantActionContext {
  store: ReturnType<typeof useSpaceStore>;
  queryClient: QueryClient;
  question: Ref<string>;
  executionTarget: Ref<ExecutionTarget | undefined>;
  threadId: Ref<string | undefined>;
  semantic: Ref<KnowledgeSearchResponse["semantic"]>;
  generation: Ref<number>;
  pendingAsk: Ref<PendingAsk | undefined>;
  pendingQuestion: Ref<string>;
  streamedAnswer: Ref<string>;
  answerStage: Ref<GroundedAnswerStage | undefined>;
  busy: ComputedRef<boolean>;
  activeScope: ComputedRef<GroundedScope>;
  activeIds: ComputedRef<string[]>;
  resetThread: () => void;
}
