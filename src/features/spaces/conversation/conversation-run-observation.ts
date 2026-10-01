import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { observeRun } from "../runs/run-observation";
import { useSpaceStore } from "../session/space-store";
import { cancelAssistantTask, getAssistantTaskError, listActiveAssistantRuns } from "./conversation-ipc";

type RunIdentity = Pick<RunObservation, "id" | "createdAt">;
type Listener = (run: RunObservation) => void;
const observed = new Map<string, Set<Listener>>();
let openConversation: ((spaceId: string, conversationId: string) => Promise<void>) | undefined;

export function registerConversationRunNavigation(
  handler: (spaceId: string, conversationId: string) => Promise<void>,
) {
  openConversation = handler;
  return () => {
    if (openConversation === handler) openConversation = undefined;
  };
}

export function observeConversationRun(
  run: RunIdentity,
  title: string,
  onUpdate?: Listener,
  location?: { spaceId: string; conversationId: string },
) {
  const tasks = useTaskStore();
  const space = useSpaceStore();
  function attachNavigation() {
    if (!location) return;
    tasks.setOpen(run.id, async () => {
      if (!openConversation) throw new Error("Assistant workspace is unavailable");
      await openConversation(location.spaceId, location.conversationId);
    });
  }
  const existing = observed.get(run.id);
  if (existing) {
    attachNavigation();
    if (onUpdate) existing.add(onUpdate);
    return () => {
      if (onUpdate) existing.delete(onUpdate);
    };
  }
  const listeners = new Set<Listener>();
  if (onUpdate) listeners.add(onUpdate);
  observed.set(run.id, listeners);
  tasks.restoreRunning({ id: run.id, title, startedAt: run.createdAt }, () => cancelAssistantTask(run.id));
  attachNavigation();
  observeRun(run.id, {
    update: (next) => {
      for (const listener of listeners) listener(next);
      if (next.completedAt !== null && space.activeSpaceId === next.homeSpaceId)
        void space.refreshActiveSpace().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
      if (next.state === "succeeded") {
        tasks.succeed(next.id);
      } else if (next.state === "cancelled") tasks.cancelled(next.id);
      else if (next.completedAt !== null) {
        void getAssistantTaskError(next.id)
          .then((reason) => tasks.fail(next.id, reason ?? i18n.global.t("spaces.assistant.tasks.failed")))
          .catch((cause: unknown) => tasks.fail(next.id, getErrorMessage(cause)));
      } else if (next.state === "paused") {
        tasks.setStatusMessage(next.id, i18n.global.t("spaces.assistant.tasks.paused"));
      } else if (next.state === "waiting") {
        tasks.setStatusMessage(next.id, i18n.global.t("spaces.assistant.tasks.waitingForResult"));
      } else tasks.setStatusMessage(next.id, "");
    },
    error: (cause) => tasks.fail(run.id, getErrorMessage(cause)),
    finished: () => {
      observed.delete(run.id);
    },
  });
  return () => {
    if (onUpdate) listeners.delete(onUpdate);
  };
}

export async function restoreConversationRuns() {
  for (const active of await listActiveAssistantRuns()) {
    observeConversationRun(active.run, active.title, undefined, {
      spaceId: active.run.homeSpaceId,
      conversationId: active.conversationId,
    });
  }
}
