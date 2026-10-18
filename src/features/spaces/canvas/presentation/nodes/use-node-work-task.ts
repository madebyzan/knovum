import { computed, type Ref } from "vue";

import { useTaskStore } from "../../../../../foundation/tasks/task-store";

/**
 * The work that is happening to one node, as the Task Center knows it.
 *
 * Work names the node it belongs to, so a node can say what is being done to it and let it be
 * stopped from where it is happening, without every feature inventing its own progress state.
 */
export function useNodeWorkTask(nodeId: Readonly<Ref<string>>) {
  const tasks = useTaskStore();
  return computed(() =>
    tasks.tasks.find(
      (task) =>
        task.subject?.kind === "node" &&
        task.subject.id === nodeId.value &&
        (task.status === "running" || task.status === "queued"),
    ),
  );
}
