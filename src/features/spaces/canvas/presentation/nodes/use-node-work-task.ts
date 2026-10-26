import { computed, type Ref } from "vue";

import type { AppTask } from "../../../../../foundation/tasks/task-model";
import { useTaskStore } from "../../../../../foundation/tasks/task-store";

/** The node a task is working on, while that work is still to finish. */
function nodeAtWork(task: AppTask): string | undefined {
  return task.subject?.kind === "node" && (task.status === "running" || task.status === "queued")
    ? task.subject.id
    : undefined;
}

/**
 * The work that is happening to one node, as the Task Center knows it.
 *
 * Work names the node it belongs to, so a node can say what is being done to it and let it be
 * stopped from where it is happening, without every feature inventing its own progress state.
 */
export function useNodeWorkTask(nodeId: Readonly<Ref<string>>) {
  const tasks = useTaskStore();
  return computed(() => tasks.tasks.find((task) => nodeAtWork(task) === nodeId.value));
}

/** Every node something is being done to right now. */
export function useNodesAtWork() {
  const tasks = useTaskStore();
  return computed(() => new Set(tasks.tasks.flatMap((task) => nodeAtWork(task) ?? [])));
}
