import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { observeRun } from "../runs/run-observation";
import { useSpaceStore } from "../session/space-store";
import {
  cancelConversationTask,
  getConversationTaskError,
  listActiveConversationRuns,
} from "./conversation-ipc";
import { conversationRunStatusKey, conversationRunTask } from "./conversation-task-projection";

type RunIdentity = Pick<RunObservation, "id" | "createdAt">;
type Listener = (run: RunObservation) => void;
const listenersByRunId = new Map<string, Set<Listener>>();
let openConversation: ((spaceId: string, conversationId: string) => Promise<void>) | undefined;

export function registerConversationRunNavigation(
  handler: (spaceId: string, conversationId: string) => Promise<void>,
) {
  openConversation = handler;
  return () => {
    if (openConversation === handler) openConversation = undefined;
  };
}

interface ConversationRunObservationInput {
  run: RunIdentity;
  title: string;
  onUpdate?: Listener | undefined;
  location?: { spaceId: string; conversationId: string };
}

export function observeConversationRun({ run, title, onUpdate, location }: ConversationRunObservationInput) {
  const tasks = useTaskStore();
  const space = useSpaceStore();
  function attachNavigation() {
    if (!location) return;
    tasks.setOpen(run.id, async () => {
      if (!openConversation) throw new Error("Conversation workspace is unavailable");
      await openConversation(location.spaceId, location.conversationId);
    });
  }
  const existing = listenersByRunId.get(run.id);
  if (existing) {
    attachNavigation();
    if (onUpdate) existing.add(onUpdate);
    return () => {
      if (onUpdate) existing.delete(onUpdate);
    };
  }
  const listeners = new Set<Listener>();
  if (onUpdate) listeners.add(onUpdate);
  listenersByRunId.set(run.id, listeners);
  attachNavigation();
  observeRun(run.id, {
    update: (next) => {
      for (const listener of listeners) listener(next);
      if (
        next.completedAt !== null &&
        next.scope.kind === "space" &&
        space.activeSpaceId === next.scope.spaceId
      )
        void space.refreshActiveSpace().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
      tasks.reconcileObserved(`conversation:${next.id}`, [
        {
          task: conversationRunTask(next, title, i18n.global.t(conversationRunStatusKey(next))),
          ...(next.completedAt === null ? { cancel: () => cancelConversationTask(next.id) } : {}),
          ...(location
            ? {
                open: async () => {
                  if (!openConversation) throw new Error("Conversation workspace is unavailable");
                  await openConversation(location.spaceId, location.conversationId);
                },
              }
            : {}),
        },
      ]);
      if (next.state === "failed" || next.state === "completed_with_errors") {
        void getConversationTaskError(next.id)
          .then((reason) => {
            const task = tasks.tasks.find(({ id }) => id === next.id);
            if (task) task.statusMessage = reason ?? i18n.global.t("spaces.conversation.tasks.failed");
            return undefined;
          })
          .catch((cause: unknown) => toast.error(getErrorMessage(cause)));
      }
    },
    error: (cause) => toast.error(getErrorMessage(cause)),
    finished: () => {
      listenersByRunId.delete(run.id);
    },
  });
  return () => {
    if (onUpdate) listeners.delete(onUpdate);
  };
}

export async function restoreConversationRuns() {
  for (const active of await listActiveConversationRuns()) {
    if (active.run.scope.kind !== "space") throw new Error("Conversation Run requires a Space");
    observeConversationRun({
      run: active.run,
      title: active.title,
      location: { spaceId: active.run.scope.spaceId, conversationId: active.conversationId },
    });
  }
}
