import { encodeExecutionTarget } from "../../../foundation/ai/execution-target-codec";
import type { ExecutionTarget } from "../../../foundation/ai/schema/models";

export type ConversationTarget = ExecutionTarget;
export const conversationTargetKey = encodeExecutionTarget;
