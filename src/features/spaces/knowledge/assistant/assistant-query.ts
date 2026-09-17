import { useQuery, type QueryClient } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import type { GroundedMessage } from "../model/grounded-message";
import type { GroundedThread } from "../model/grounded-thread";
import { listGroundedMessages, listGroundedThreads } from "./grounded-repository";

export const knowledgeAssistantQueryKeys = {
  threads: (spaceId: string) => ["spaces", "knowledge-assistant", "threads", spaceId] as const,
  messages: (threadId: string) => ["spaces", "knowledge-assistant", "messages", threadId] as const,
};

export function useKnowledgeAssistantQueries(
  spaceId: MaybeRefOrGetter<string | undefined>,
  threadId: MaybeRefOrGetter<string | undefined>,
) {
  const threads = useQuery<GroundedThread[]>({
    queryKey: computed(() => knowledgeAssistantQueryKeys.threads(toValue(spaceId) ?? "")),
    enabled: computed(() => Boolean(toValue(spaceId))),
    queryFn: () => listGroundedThreads(requiredId(toValue(spaceId), "Space")),
  });
  const messages = useQuery<GroundedMessage[]>({
    queryKey: computed(() => knowledgeAssistantQueryKeys.messages(toValue(threadId) ?? "")),
    enabled: computed(() => Boolean(toValue(threadId))),
    queryFn: () => listGroundedMessages(requiredId(toValue(threadId), "Knowledge thread")),
  });
  return { threads, messages };
}

export function cacheKnowledgeThreads(client: QueryClient, spaceId: string, threads: GroundedThread[]) {
  client.setQueryData(knowledgeAssistantQueryKeys.threads(spaceId), threads);
}

export function appendKnowledgeMessages(
  client: QueryClient,
  threadId: string,
  messages: readonly GroundedMessage[],
) {
  client.setQueryData<GroundedMessage[]>(knowledgeAssistantQueryKeys.messages(threadId), (current) => {
    const byId = new Map((current ?? []).map((message) => [message.id, message]));
    for (const message of messages) byId.set(message.id, message);
    return [...byId.values()];
  });
}

function requiredId(value: string | undefined, kind: string): string {
  if (!value) throw new Error(`${kind} query requires an active identifier`);
  return value;
}
