import { shallowRef, watch, type Ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunEvent } from "../contracts/generated/RunEvent";
import { listConversationRunEvents } from "./conversation-ipc";

export function useConversationActivities(
  selectedConversationId: Ref<string>,
  activeRunId: Ref<string | undefined>,
  revision: Ref<number>,
) {
  const eventsByRun = shallowRef<Record<string, RunEvent[]>>({});
  const error = shallowRef<string>();
  let selection = 0;
  const pending = new Map<string, Promise<void>>();

  async function loadPage(runId: string, events: RunEvent[], current: number): Promise<void> {
    const seen = events.at(-1)?.sequence ?? 0;
    const page = await listConversationRunEvents(runId, seen);
    if (selection !== current) return;
    events.push(...page.filter((event) => event.sequence > seen));
    if (page.length === 1000) await loadPage(runId, events, current);
  }

  async function load(runId: string) {
    const current = selection;
    const events = [...(eventsByRun.value[runId] ?? [])];
    await loadPage(runId, events, current);
    if (selection !== current) return;
    eventsByRun.value = { ...eventsByRun.value, [runId]: events };
    error.value = undefined;
  }

  function refresh(runId: string) {
    const current = selection;
    const previous = pending.get(runId) ?? Promise.resolve();
    const next = previous
      .then(() => load(runId))
      .catch((cause: unknown) => {
        if (selection === current) error.value = getErrorMessage(cause);
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
