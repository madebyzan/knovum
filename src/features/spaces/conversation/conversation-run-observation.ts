import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { RunObservation } from "../contracts/generated/RunObservation";
import { createRunAudience, type RunListener } from "../runs/run-audience";
import { observeRun } from "../runs/run-observation";
import { useSpaceStore } from "../session/space-store";
import { listActiveConversationRuns } from "./conversation-ipc";
import { conversationRunTask } from "./conversation-task-projection";

type RunIdentity = Pick<RunObservation, "id" | "createdAt">;
const audience = createRunAudience();

interface ConversationRunObservationInput {
  run: RunIdentity;
  title: string;
  onUpdate?: RunListener | undefined;
}

export function observeConversationRun({ run, title, onUpdate }: ConversationRunObservationInput) {
  const tasks = useTaskStore();
  const space = useSpaceStore();
  const { listeners, release } = audience.join(run.id, onUpdate);
  if (!listeners) return release;
  observeRun(run.id, {
    update: (next) => {
      for (const listener of listeners) listener(next);
      if (
        next.completedAt !== null &&
        next.scope.kind === "space" &&
        space.activeSpaceId === next.scope.spaceId
      )
        void space.refreshActiveSpace().catch((cause: unknown) => toast.error(getErrorMessage(cause)));
      // Recorded as work in flight, not as a row: the conversation reports this Run, an update
      // restart may not interrupt it.
      tasks.reconcileObserved(`conversation:${next.id}`, [{ task: conversationRunTask(next, title) }]);
    },
    error: (cause) => toast.error(getErrorMessage(cause)),
    finished: () => audience.leave(run.id),
  });
  return release;
}

export async function restoreConversationRuns() {
  for (const active of await listActiveConversationRuns()) {
    if (active.run.scope.kind !== "space") throw new Error("Conversation Run requires a Space");
    observeConversationRun({ run: active.run, title: active.title });
  }
}
