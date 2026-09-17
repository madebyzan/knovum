import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { useSpaceStore } from "../session/space-store";
import { yieldToCanvas } from "./canvas-yield";
import { beginPendingDerivation, endPendingDerivation } from "./pending-session";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export interface RunDerivationOptions<Output> {
  store: SpaceStore;
  pendingNode: SpaceCanvasNode;
  sourceNodeIds: string[];
  task?: {
    id?: string;
    title: string;
    detail?: string;
    cancel?: () => Promise<void> | void;
  };
  failureMessageKey?: string;
  execute: (context: { spaceId: string; pendingId: string; taskId?: string | undefined }) => Promise<Output>;
  onSuccess?: (completed: Output, spaceId: string) => Promise<void> | void;
  onError?: (error: unknown, spaceId: string) => Promise<void> | void;
}

export async function runDerivation<Output>(
  options: RunDerivationOptions<Output>,
): Promise<Output | undefined> {
  const { store, pendingNode, sourceNodeIds, task, execute, onSuccess, onError, failureMessageKey } = options;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return undefined;

  beginPendingDerivation(pendingNode.id, sourceNodeIds);
  const tasks = useTaskStore();
  const taskId = task ? tasks.start(task.title, task.detail, task.cancel, task.id) : undefined;

  try {
    await yieldToCanvas();
    const completed = await execute({ spaceId, pendingId: pendingNode.id, taskId });
    if (onSuccess) {
      await onSuccess(completed, spaceId);
    }
    if (taskId) {
      tasks.succeed(taskId);
    }
    return completed;
  } catch (error) {
    if (onError) {
      await onError(error, spaceId);
    } else {
      await store.refreshActiveSpace();
    }
    if (taskId) {
      tasks.fail(taskId, String(error));
    }
    if (failureMessageKey) {
      toast.error(`${i18n.global.t(failureMessageKey)}: ${String(error)}`);
    }
    return undefined;
  } finally {
    endPendingDerivation(pendingNode.id);
  }
}
