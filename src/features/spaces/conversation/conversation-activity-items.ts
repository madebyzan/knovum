import { z } from "zod";

import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { RunEvent } from "../contracts/generated/RunEvent";

export type ActivityItem = {
  id: string;
  title: string;
  status: string;
  context?: unknown;
  output?: unknown;
  outputName?: string;
};

const toolPayload = z.strictObject({
  toolCallId: z.string(),
  title: z.string().nullish(),
  actionId: z.string().nullish(),
  status: z.enum(["pending", "in_progress", "completed", "failed", "outcome_unknown"]).nullish(),
  context: z.unknown().optional(),
});
const textPayload = z.object({ text: z.string() });
const toolEvents = new Set([
  "external_agent_tool_started",
  "external_agent_tool_updated",
  "conversation_tool_started",
  "conversation_tool_finished",
]);

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

type ActivityState = { items: ActivityItem[]; indexByCallId: Map<string, number>; commentary: string };
type ToolLabels = { action: (actionId: string) => string; default: string };

function appendCommentary(state: ActivityState, event: RunEvent) {
  if (!state.commentary.trim()) return;
  state.items.push({ id: `text:${event.sequence}`, title: state.commentary.trim(), status: "commentary" });
  state.commentary = "";
}

function toolTitle(
  payload: z.infer<typeof toolPayload>,
  previous: ActivityItem | undefined,
  labels: ToolLabels,
) {
  return (
    payload.title ?? previous?.title ?? (payload.actionId ? labels.action(payload.actionId) : labels.default)
  );
}

function toolStatus(event: RunEvent, status: z.infer<typeof toolPayload>["status"], previous?: ActivityItem) {
  return (
    status ??
    (event.kind === "conversation_tool_finished" ? "completed" : (previous?.status ?? "in_progress"))
  );
}

function appendTool(
  state: ActivityState,
  event: RunEvent,
  outputByCallId: Map<string, { name: string; value: unknown }>,
  labels: ToolLabels,
) {
  if (!toolEvents.has(event.kind)) return;
  const parsed = toolPayload.safeParse(event.payload);
  if (!parsed.success) return;
  appendCommentary(state, event);
  const { toolCallId, context } = parsed.data;
  const index = state.indexByCallId.get(toolCallId);
  const previous = index === undefined ? undefined : state.items[index];
  const output = outputByCallId.get(toolCallId);
  const item: ActivityItem = {
    id: toolCallId,
    title: toolTitle(parsed.data, previous, labels),
    status: toolStatus(event, parsed.data.status, previous),
    context: context == null ? previous?.context : context,
    output: output?.value,
    ...(output && { outputName: output.name }),
  };
  if (index === undefined) {
    state.indexByCallId.set(toolCallId, state.items.length);
    state.items.push(item);
  } else state.items[index] = item;
}

interface ConversationActivityInput {
  events: readonly RunEvent[];
  messages: readonly ConversationMessage[];
  runId: string;
  labels: ToolLabels;
}

export function conversationActivityItems({ events, messages, runId, labels }: ConversationActivityInput) {
  const outputByCallId = toolOutputs(messages, runId);
  const state: ActivityState = { items: [], indexByCallId: new Map(), commentary: "" };
  for (const event of events) {
    if (event.kind === "external_agent_message") {
      const parsed = textPayload.safeParse(event.payload);
      if (parsed.success) state.commentary += parsed.data.text;
    } else appendTool(state, event, outputByCallId, labels);
  }
  return state.items;
}
