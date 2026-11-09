import { CircleAlert, CircleCheck, CircleStop, CircleX, Clock3 } from "@lucide/vue";

import type { AppTask, TaskStatus } from "../../foundation/tasks/task-model";

export const STATUS_LABEL_KEYS = {
  queued: "tasks.status.queued",
  running: "tasks.status.running",
  attention: "tasks.status.attention",
  succeeded: "tasks.status.succeeded",
  failed: "tasks.status.failed",
  cancelled: "tasks.status.cancelled",
} satisfies Record<TaskStatus, string>;

export function isActiveTask(status: TaskStatus): boolean {
  return status === "queued" || status === "running";
}

export function canCancelTask(status: TaskStatus): boolean {
  return isActiveTask(status) || status === "attention";
}

export function isTerminalTask(status: TaskStatus): boolean {
  return status === "succeeded" || status === "cancelled";
}

export function taskStatusClass(status: TaskStatus): string {
  if (status === "failed") return "text-fg-danger";
  if (status === "succeeded") return "text-fg-success";
  if (status === "running") return "text-fg-accent";
  return "text-fg-tertiary";
}

export function taskStatusIcon(status: TaskStatus) {
  switch (status) {
    case "queued":
      return Clock3;
    case "attention":
      return CircleAlert;
    case "succeeded":
      return CircleCheck;
    case "cancelled":
      return CircleStop;
    case "failed":
    default:
      return CircleX;
  }
}

export function taskStatusIconClass(status: TaskStatus): string {
  switch (status) {
    case "succeeded":
      return "text-fg-success";
    case "failed":
      return "text-fg-danger";
    case "attention":
    case "cancelled":
      return "text-fg-secondary";
    case "queued":
    default:
      return "text-fg-tertiary";
  }
}

export function taskSubtitle(task: AppTask): string | undefined {
  const parts: string[] = [];
  if (task.scope?.label) {
    parts.push(task.scope.label);
  }
  if (task.subject?.label) {
    parts.push(
      task.subject.typeLabel ? `${task.subject.typeLabel} · ${task.subject.label}` : task.subject.label,
    );
  }
  return parts.length > 0 ? parts.join(" · ") : undefined;
}
