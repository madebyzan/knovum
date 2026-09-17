import { computed, ref, shallowRef } from "vue";

import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import type { useSpaceStore } from "../../session/space-store";
import type { GroundedScope } from "../model/grounded-scope";
import type { KnowledgeSearchResponse } from "../model/knowledge-search-response";
import { useKnowledgeAssistantQueries } from "./assistant-query";
import type { GroundedAnswerStage } from "./grounded-assistant";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export type PendingAsk = { generation: number; spaceId: string; jobId: string };

export function useSpaceKnowledgeAssistantSession(store: SpaceStore) {
  const scope = ref<GroundedScope>("space");
  const question = ref("");
  const executionTarget = ref<ExecutionTarget>();
  const threadId = ref<string>();
  const semantic = ref<KnowledgeSearchResponse["semantic"]>("empty");
  const generation = ref(0);
  const pendingAsk = shallowRef<PendingAsk>();
  const pendingQuestion = ref("");
  const streamedAnswer = ref("");
  const answerStage = ref<GroundedAnswerStage>();
  const queries = useKnowledgeAssistantQueries(() => store.activeSpace?.id, threadId);
  const threads = computed(() => queries.threads.data.value ?? []);
  const messages = computed(() => queries.messages.data.value ?? []);
  const busy = computed(
    () =>
      pendingAsk.value?.generation === generation.value && pendingAsk.value.spaceId === store.activeSpace?.id,
  );
  return {
    scope,
    question,
    executionTarget,
    threadId,
    semantic,
    generation,
    pendingAsk,
    pendingQuestion,
    streamedAnswer,
    answerStage,
    threads,
    messages,
    busy,
  };
}
