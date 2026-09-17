import { useQueryClient } from "@tanstack/vue-query";
import { computed, type ComputedRef } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import type { KnowledgeAssistantActionContext } from "./action-context";
import { saveAsKnowledge } from "./answer-capture";
import { askKnowledge, cancelKnowledgeAsk, deleteThread, renameThread } from "./assistant-actions";
import { useKnowledgeAssistantModel } from "./assistant-model";
import { useSpaceKnowledgeAssistantSession } from "./use-assistant-session";
import { useSpaceKnowledgeScopeRequests } from "./use-scope-requests";

export function useSpaceKnowledgeAssistant() {
  const store = useSpaceStore();
  const { t } = useI18n();
  const queryClient = useQueryClient();
  const session = useSpaceKnowledgeAssistantSession(store);
  const { scope, threadId, semantic, generation, threads, messages } = session;
  const model = useKnowledgeAssistantModel({
    store,
    threads,
    messages,
    threadId,
    scope,
    semantic,
    generation,
  });
  const threadOptions = computed(() => [
    { value: "new", label: t("spaces.assistant.newKnowledgeThread") },
    ...threads.value.map((thread) => ({ value: thread.id, label: thread.title })),
  ]);
  const selectedThread = computed({ get: () => threadId.value ?? "new", set: model.selectThread });
  const actions = actionContext({ store, queryClient, session, model });
  useSpaceKnowledgeScopeRequests({
    store,
    threadId,
    messages,
    scope,
    resetThread: model.resetThread,
    selectedContentIds: model.selectedContentIds,
    sectionIds: model.sectionIds,
    contentNodes: model.contentNodes,
  });

  return knowledgeAssistantApi({
    store,
    session,
    model,
    threadOptions,
    selectedThread,
    actions,
  });
}

type SpaceStore = ReturnType<typeof useSpaceStore>;
type AssistantSession = ReturnType<typeof useSpaceKnowledgeAssistantSession>;
type AssistantModel = ReturnType<typeof useKnowledgeAssistantModel>;

interface AssistantActionInput {
  store: SpaceStore;
  queryClient: ReturnType<typeof useQueryClient>;
  session: AssistantSession;
  model: AssistantModel;
}

interface KnowledgeAssistantApiInput {
  store: SpaceStore;
  session: AssistantSession;
  model: AssistantModel;
  threadOptions: ComputedRef<{ value: string; label: string }[]>;
  selectedThread: ComputedRef<string>;
  actions: KnowledgeAssistantActionContext;
}

function actionContext(input: AssistantActionInput): KnowledgeAssistantActionContext {
  return {
    store: input.store,
    queryClient: input.queryClient,
    ...input.session,
    activeScope: input.model.activeScope,
    activeIds: input.model.activeIds,
    resetThread: input.model.resetThread,
  };
}

function knowledgeAssistantApi(input: KnowledgeAssistantApiInput) {
  const { session, model, actions } = input;
  return {
    store: input.store,
    ...session,
    activeThread: model.activeThread,
    activeScope: model.activeScope,
    scopeCounts: model.scopeCounts,
    scopeTitles: model.scopeTitles,
    threadOptions: input.threadOptions,
    selectedThread: input.selectedThread,
    askKnowledge: (value?: string) => askKnowledge(actions, value),
    cancelAsk: () => cancelKnowledgeAsk(actions),
    startNewThread: model.resetThread,
    renameThread: (title: string) => renameThread(actions, title),
    deleteThread: () => deleteThread(actions),
    saveAsKnowledge: (message: Parameters<typeof saveAsKnowledge>[1]) => saveAsKnowledge(actions, message),
  };
}
