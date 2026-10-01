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
}
