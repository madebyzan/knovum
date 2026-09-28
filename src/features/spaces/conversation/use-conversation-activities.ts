import { shallowRef, watch, type Ref } from "vue";

import type { RunEvent } from "../contracts/generated/RunEvent";
import { listAssistantRunEvents } from "./conversation-ipc";

export function useConversationActivities(
  selectedConversationId: Ref<string>,
  activeRunId: Ref<string | undefined>,
  revision: Ref<number>,
) {
  const eventsByRun = shallowRef<Record<string, RunEvent[]>>({});
  const error = shallowRef<string>();
  let selection = 0;
  const pending = new Map<string, Promise<void>>();

  async function load(runId: string) {
    const current = selection;
    let events = eventsByRun.value[runId] ?? [];
    let page: RunEvent[];
    do {
      page = await listAssistantRunEvents(runId, events.at(-1)?.sequence ?? 0);
      if (selection !== current) return;
      const seen = events.at(-1)?.sequence ?? 0;
      events = [...events, ...page.filter((event) => event.sequence > seen)];
    } while (page.length === 1000);
    eventsByRun.value = { ...eventsByRun.value, [runId]: events };
    error.value = undefined;
  }

  function refresh(runId: string) {
    const current = selection;
    const previous = pending.get(runId) ?? Promise.resolve();
    const next = previous.then(() => load(runId)).catch((cause: unknown) => {
      if (selection === current) error.value = String(cause);
    });
    pending.set(runId, next);
    void next.finally(() => {
      if (pending.get(runId) === next) pending.delete(runId);
    });
  }

  watch(selectedConversationId, () => {
    selection += 1;
    eventsByRun.value = {};
    error.value = undefined;
  });
  watch(revision, () => {
    if (activeRunId.value && eventsByRun.value[activeRunId.value] !== undefined) refresh(activeRunId.value);
  });

  function inspect(runId: string) {
    if (eventsByRun.value[runId] === undefined) {
      eventsByRun.value = { ...eventsByRun.value, [runId]: [] };
      refresh(runId);
    }
  }

  return { eventsByRun, error, inspect };
}
