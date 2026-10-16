import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { yieldToCanvas } from "./canvas-yield";

interface DerivationTask {
  id?: string;
  title: string;
  statusMessage?: string;
  cancel?: () => Promise<void> | void;
}

interface DerivationExecutionContext {
  spaceId: string;
  taskId?: string | undefined;
}

export interface RunDerivationOptions<Output> {
  spaceId: string;
  task: DerivationTask;
  failureMessageKey: string;
  execute: (context: DerivationExecutionContext) => Promise<Output>;
  onSuccess: (completed: Output, spaceId: string) => Promise<void> | void;
  onError: (error: unknown, spaceId: string) => Promise<void> | void;
}

export async function runDerivation<Output>(
  options: RunDerivationOptions<Output>,
): Promise<Output | undefined> {
  const { spaceId, task, execute, onSuccess, onError, failureMessageKey } = options;

  const tasks = useTaskStore();
  const taskId = tasks.start(task.title, task.statusMessage, task.cancel, task.id);

  let completed: Output;
  try {
    await yieldToCanvas();
    completed = await execute({ spaceId, taskId });
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
