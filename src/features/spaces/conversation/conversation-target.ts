import type { AgentKind } from "../../../foundation/ai/agent-models";
import type { ExecutionTarget } from "../../../foundation/ai/ai-models";

export type ConversationTarget = ExecutionTarget | { type: "agent"; runtimeKind: AgentKind };

export function conversationTargetKey(target: ConversationTarget): string {
  return target.type === "agent"
    ? `agent:${target.runtimeKind}`
    : `model:${target.connectionId}:${target.modelId}`;
}
