import { shallowRef, watch, type Ref } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { RunEvent } from "../contracts/generated/RunEvent";
import { listConversationRunEvents } from "./conversation-ipc";

export function useConversationActivities(
  selectedConversationId: Ref<string>,
  activeRunId: Ref<string | undefined>,
  revision: Ref<number>,
) {
  const eventsByRunId = shallowRef<Record<string, RunEvent[]>>({});
  const error = shallowRef<string>();
  let selection = 0;
  const pendingLoadByRunId = new Map<string, Promise<void>>();

  async function loadPage(runId: string, events: RunEvent[], current: number): Promise<void> {
    const seen = events.at(-1)?.sequence ?? 0;
    const page = await listConversationRunEvents(runId, seen);
    if (selection !== current) return;
    events.push(...page.filter((event) => event.sequence > seen));
    if (page.length === 1000) await loadPage(runId, events, current);
  }

  async function load(runId: string) {
    const current = selection;
    const events = [...(eventsByRunId.value[runId] ?? [])];
    await loadPage(runId, events, current);
    if (selection !== current) return;
    eventsByRunId.value = { ...eventsByRunId.value, [runId]: events };
    error.value = undefined;
  }

  function refresh(runId: string): Promise<void> {
    const current = selection;
    const previous = pendingLoadByRunId.get(runId) ?? Promise.resolve();
    const next = previous
      .then(() => load(runId))
      .catch((cause: unknown) => {
        if (selection === current) error.value = getErrorMessage(cause);
      });
    pendingLoadByRunId.set(runId, next);
    void next.finally(() => {
      if (pendingLoadByRunId.get(runId) === next) pendingLoadByRunId.delete(runId);
    });
    return next;
  }

  watch(selectedConversationId, () => {
    selection += 1;
    eventsByRunId.value = {};
    error.value = undefined;
  });
  watch(revision, () => {
    if (activeRunId.value && eventsByRunId.value[activeRunId.value] !== undefined) refresh(activeRunId.value);
  });

  function inspect(runId: string) {
    if (eventsByRunId.value[runId] === undefined) void follow(runId);
  }

  /** Brings a run's ledger up to date, starting to follow the run if nothing has read it yet. */
  function follow(runId: string) {
    if (eventsByRunId.value[runId] === undefined)
      eventsByRunId.value = { ...eventsByRunId.value, [runId]: [] };
    return refresh(runId);
  }

  return { eventsByRunId, activityError: error, inspectActivity: inspect, loadActivity: follow };
}
