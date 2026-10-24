import type { AppTask, TaskStatus } from "../../../foundation/tasks/task-model";
import type { RunObservation } from "../contracts/generated/RunObservation";
import type { RunState } from "../contracts/generated/RunState";

const TASK_STATUS_BY_RUN_STATE: Record<RunState, TaskStatus> = {
  queued: "queued",
  running: "running",
  waiting: "attention",
  paused: "attention",
  succeeded: "succeeded",
  completed_with_errors: "failed",
  failed: "failed",
  cancelled: "cancelled",
};

/**
 * A conversation Run as work in flight. It is deliberately unlisted: the conversation it answers
 * already shows that it is running, lets it be stopped and reports how it ended, so a Task Center
 * row would be a second copy of all of it. What the Task Center is still needed for is the fact of
 * the work — an update restart must not land in the middle of an answer.
 */
export function conversationRunTask(run: RunObservation, title: string): AppTask {
  return {
    id: run.id,
    title,
    status: run.waitReason === "executor" ? "running" : TASK_STATUS_BY_RUN_STATE[run.state],
    startedAt: run.createdAt,
    listed: false,
  };
}
