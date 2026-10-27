import { z } from "zod";

import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { RunEvent } from "../contracts/generated/RunEvent";

export type ActivityItem = {
  id: string;
  title: string;
  status: ToolStatus;
  context?: unknown;
  output?: unknown;
  outputName?: string;
  progress?: ToolProgress;
};

const toolPayload = z.strictObject({
  toolCallId: z.string(),
  title: z.string().nullish(),
  actionId: z.string().nullish(),
  status: z.enum(["pending", "in_progress", "completed", "failed", "outcome_unknown"]).nullish(),
  context: z.unknown().optional(),
});
const progressPayload = z.strictObject({
  toolCallId: z.string(),
  progress: z.number(),
  total: z.number().nullish(),
  message: z.string().nullish(),
});
export type ToolProgress = Omit<z.infer<typeof progressPayload>, "toolCallId">;
type ToolPayload = z.infer<typeof toolPayload>;
type ToolStatus = NonNullable<ToolPayload["status"]>;
type ToolLabels = { action: (actionId: string) => string; default: string };

const toolEvents = new Set<RunEvent["kind"]>([
  "external_agent_tool_started",
  "external_agent_tool_updated",
  "conversation_tool_started",
  "conversation_tool_finished",
]);

/** The call is still under way. */
export function isRunning({ status }: ActivityItem) {
  return status === "pending" || status === "in_progress";
}

/**
 * The call did not end well: it failed, its outcome is unknown, or the run stopped while it was
 * still under way. While the run is producing, a call under way is simply running.
 */
export function isUnfinished(item: ActivityItem, active: boolean) {
  return item.status === "failed" || item.status === "outcome_unknown" || (!active && isRunning(item));
}

/** The call a ledger event reports on, when it is a tool event in the vocabulary this reads. */
function toolPayloadOf(event: RunEvent): ToolPayload | undefined {
  if (!toolEvents.has(event.kind)) return undefined;
  const parsed = toolPayload.safeParse(event.payload);
  return parsed.success ? parsed.data : undefined;
}

export function toolCallIdOf(event: RunEvent): string | undefined {
  return toolPayloadOf(event)?.toolCallId;
}

function toolOutputs(messages: readonly ConversationMessage[], runId: string) {
  const outputByCallId = new Map<string, { name: string; value: unknown }>();
  for (const message of messages) {
    if (message.role !== "tool" || message.content.runId !== runId) continue;
    for (const part of message.content.parts) {
      if (part.type === "tool_result")
        outputByCallId.set(part.callId, { name: part.name, value: part.result });
    }
  }
  return outputByCallId;
}

function toolStatus(event: RunEvent, payload: ToolPayload, previous: ActivityItem | undefined): ToolStatus {
  return (
    payload.status ??
    (event.kind === "conversation_tool_finished" ? "completed" : (previous?.status ?? "in_progress"))
  );
}

function nextItem(
  event: RunEvent,
  payload: ToolPayload,
  previous: ActivityItem | undefined,
  labels: ToolLabels,
): ActivityItem {
  return {
    id: payload.toolCallId,
    title:
      payload.title ??
      previous?.title ??
      (payload.actionId ? labels.action(payload.actionId) : labels.default),
    status: toolStatus(event, payload, previous),
    context: payload.context == null ? previous?.context : payload.context,
  };
}

interface ConversationToolInput {
  events: readonly RunEvent[];
  messages: readonly ConversationMessage[];
  runId: string;
  labels: ToolLabels;
}

/**
 * Every tool a run called, by call id, in the order the ledger first saw each: what the call
 * was, how it went, what it returned. Where a call sits among the words of the answer is the
 * transcript's business; this only says what each call is.
 */
export function conversationToolItems({
  events,
  messages,
  runId,
  labels,
}: ConversationToolInput): ReadonlyMap<string, ActivityItem> {
  const items = new Map<string, ActivityItem>();
  const progressByCallId = new Map<string, ToolProgress>();
  for (const event of events) {
    if (event.kind === "mcp_progress") {
      const parsed = progressPayload.safeParse(event.payload);
      if (parsed.success) {
        const { toolCallId, ...progress } = parsed.data;
        progressByCallId.set(toolCallId, progress);
      }
      continue;
    }
    const payload = toolPayloadOf(event);
    if (payload)
      items.set(payload.toolCallId, nextItem(event, payload, items.get(payload.toolCallId), labels));
  }
  for (const [callId, output] of toolOutputs(messages, runId)) {
    const item = items.get(callId);
    if (item) items.set(callId, { ...item, output: output.value, outputName: output.name });
  }
  for (const [callId, progress] of progressByCallId) {
    const item = items.get(callId);
    if (item) items.set(callId, { ...item, progress });
  }
  return items;
}
