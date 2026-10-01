import type { Ref } from "vue";

import type { AppTask } from "./task-model";

const TASK_HISTORY_LIMIT = 50;

export function trimTaskHistory(tasks: Ref<AppTask[]>, remove: (id: string) => void) {
  const terminal = tasks.value.filter((task) => task.status === "succeeded" || task.status === "cancelled");
  for (const { id } of terminal.slice(TASK_HISTORY_LIMIT)) remove(id);
}
