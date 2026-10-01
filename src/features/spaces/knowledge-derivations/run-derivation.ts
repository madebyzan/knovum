import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { useSpaceStore } from "../session/space-store";
import { yieldToCanvas } from "./canvas-yield";

type SpaceStore = Pick<ReturnType<typeof useSpaceStore>, "activeSpaceId">;

export interface RunDerivationOptions<Output> {
  store: SpaceStore;
  pendingId: string;
  task: {
    id?: string;
    title: string;
    statusMessage?: string;
    cancel?: () => Promise<void> | void;
  };
  failureMessageKey: string;
  execute: (context: { spaceId: string; pendingId: string; taskId?: string | undefined }) => Promise<Output>;
  onSuccess: (completed: Output, spaceId: string) => Promise<void> | void;
  onError: (error: unknown, spaceId: string) => Promise<void> | void;
}

export async function runDerivation<Output>(
  options: RunDerivationOptions<Output>,
): Promise<Output | undefined> {
  const { store, pendingId, task, execute, onSuccess, onError, failureMessageKey } = options;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return undefined;

  const tasks = useTaskStore();
  const taskId = tasks.start(task.title, task.statusMessage, task.cancel, task.id);

  let completed: Output;
  try {
    await yieldToCanvas();
    completed = await execute({ spaceId, pendingId, taskId });
  } catch (error) {
    tasks.fail(taskId, getErrorMessage(error));
    try {
      await onError(error, spaceId);
    } catch (recoveryError) {
      throw new AggregateError([error, recoveryError], "Derivation and recovery failed", { cause: error });
    }
    toast.error(`${i18n.global.t(failureMessageKey)}: ${getErrorMessage(error)}`);
    return undefined;
  }

  try {
    await onSuccess(completed, spaceId);
  } finally {
    tasks.succeed(taskId);
  }
  return completed;
}
