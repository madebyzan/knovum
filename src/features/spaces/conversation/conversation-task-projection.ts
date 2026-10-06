import type { AppTask, TaskStatus } from "../../../foundation/tasks/task-model";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { RunState } from "../contracts/generated/RunState";

const statuses: Record<RunState, TaskStatus> = {
  queued: "queued",
  running: "running",
  waiting: "attention",
  paused: "attention",
  succeeded: "succeeded",
  completed_with_errors: "failed",
  failed: "failed",
  cancelled: "cancelled",
};

export function conversationRunStatusKey(run: Pick<RunObservation, "state" | "waitReason">) {
  switch (run.waitReason) {
    case "model_outcome_unknown":
      return "spaces.conversation.tasks.modelOutcomeUnknown";
    case "model_response_uncommitted":
      return "spaces.conversation.tasks.modelResponseUncommitted";
    case "model_usage_unknown":
      return "spaces.conversation.tasks.modelUsageUnknown";
    default:
      return run.state === "paused"
        ? "spaces.conversation.tasks.paused"
        : "spaces.conversation.tasks.waitingForResult";
  }
}

export function conversationRunTask(run: RunObservation, title: string, statusMessage: string): AppTask {
  return {
    id: run.id,
    title,
    status: run.waitReason === "executor" ? "running" : statuses[run.state],
    startedAt: run.createdAt,
    ...(run.state === "waiting" || run.state === "paused" ? { statusMessage } : {}),
  };
}
