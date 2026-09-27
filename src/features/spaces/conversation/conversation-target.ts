import type { AgentKind } from "../../../foundation/ai/schema/agents";
import type { ExecutionTarget } from "../../../foundation/ai/schema/models";

export type ConversationTarget = ExecutionTarget | { type: "agent"; runtimeKind: AgentKind };

export function conversationTargetKey(target: ConversationTarget): string {
  return target.type === "agent"
    ? `agent:${target.runtimeKind}`
    : `model:${target.connectionId}:${target.modelId}`;
}
