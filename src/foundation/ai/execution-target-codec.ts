import type { TaskTargetRef } from "../../shared/contracts/generated/TaskTargetRef";
import { executionTargetSchema } from "./schema/models";

export const encodeExecutionTarget = (target: TaskTargetRef) => JSON.stringify(target);

export function decodeExecutionTarget(value: string): TaskTargetRef {
  const parsed: unknown = JSON.parse(value);
  return executionTargetSchema.parse(parsed);
}
