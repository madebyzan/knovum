import type { TaskReference } from "../../shared/contracts/task-reference";

export type TaskStatus = "queued" | "running" | "attention" | "succeeded" | "failed" | "cancelled";

export interface AppTask {
  id: string;
  title: string;
  status: TaskStatus;
  progress?: number;
  statusMessage?: string;
  startedAt: number;
  cancellable?: boolean;
  cancelling?: boolean;
  retryable?: boolean;
  openable?: boolean;
  producer?: string;
  durable?: boolean;
  scope?: TaskReference;
  subject?: TaskReference;
  /**
   * Whether the Task Center shows this work. It lists what has nowhere else to be watched; work
   * with its own surface — a conversation Run, watched in the conversation it is answering, with
   * its own status, stop button and failure — would only be a second, worse copy of that surface.
   *
   * Unlisted work is still work in flight: it counts as running, so an update restart waits for it.
   */
  listed?: boolean;
}
