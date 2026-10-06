import type { TaskTargetRef } from "../../shared/contracts/generated/TaskTargetRef";

export const encodeExecutionTarget = (target: TaskTargetRef) => JSON.stringify(target);
