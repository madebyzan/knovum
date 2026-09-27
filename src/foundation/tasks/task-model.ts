export type TaskStatus = "running" | "succeeded" | "failed" | "cancelled";

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
}
