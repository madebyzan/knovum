import { executionTargetSchema, type ExecutionTarget } from "./schema/models";

export const encodeExecutionTarget = (target: ExecutionTarget) => JSON.stringify(target);

export function decodeExecutionTarget(value: string): ExecutionTarget {
  const parsed: unknown = JSON.parse(value);
  return executionTargetSchema.parse(parsed);
}
