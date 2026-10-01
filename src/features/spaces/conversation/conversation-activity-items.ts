import { z } from "zod";

import type { AssistantMessage } from "../contracts/generated/AssistantMessage";
import type { RunEvent } from "../contracts/generated/RunEvent";

export type ActivityItem = {
  id: string;
  title: string;
  status: string;
  detail?: unknown;
  result?: unknown;
  resultName?: string;
};

const toolPayload = z.object({
  toolCallId: z.string(),
  title: z.string().nullish(),
  actionId: z.string().nullish(),
  status: z.enum(["pending", "in_progress", "completed", "failed"]).nullish(),
  detail: z.unknown().optional(),
});
const textPayload = z.object({ text: z.string() });
const toolEvents = new Set([
  "external_agent_tool_started",
  "external_agent_tool_updated",
  "assistant_tool_started",
  "assistant_tool_finished",
]);

function toolResults(messages: readonly AssistantMessage[], runId: string) {
  const results = new Map<string, { name: string; value: unknown }>();
  for (const message of messages) {
    if (message.role !== "tool" || message.content.runId !== runId) continue;
    for (const part of message.content.parts) {
      if (part.type === "tool_result") results.set(part.callId, { name: part.name, value: part.result });
    }
  }
  return results;
}

type ActivityState = { items: ActivityItem[]; positions: Map<string, number>; commentary: string };
type ToolLabels = { action: (id: string) => string; default: string };

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
    status ?? (event.kind === "assistant_tool_finished" ? "completed" : (previous?.status ?? "in_progress"))
  );
}

function appendTool(
  state: ActivityState,
  event: RunEvent,
  results: Map<string, { name: string; value: unknown }>,
  labels: ToolLabels,
) {
  if (!toolEvents.has(event.kind)) return;
  const parsed = toolPayload.safeParse(event.payload);
  if (!parsed.success) return;
  appendCommentary(state, event);
  const { toolCallId, detail } = parsed.data;
  const index = state.positions.get(toolCallId);
  const previous = index === undefined ? undefined : state.items[index];
  const result = results.get(toolCallId);
  const item: ActivityItem = {
    id: toolCallId,
    title: toolTitle(parsed.data, previous, labels),
    status: toolStatus(event, parsed.data.status, previous),
    detail: detail == null ? previous?.detail : detail,
    result: result?.value,
    ...(result && { resultName: result.name }),
  };
  if (index === undefined) {
    state.positions.set(toolCallId, state.items.length);
    state.items.push(item);
  } else state.items[index] = item;
}

export function conversationActivityItems(
  events: readonly RunEvent[],
  messages: readonly AssistantMessage[],
  runId: string,
  labels: ToolLabels,
) {
  const results = toolResults(messages, runId);
  const state: ActivityState = { items: [], positions: new Map(), commentary: "" };
  for (const event of events) {
    if (event.kind === "external_agent_message") {
      const parsed = textPayload.safeParse(event.payload);
      if (parsed.success) state.commentary += parsed.data.text;
    } else appendTool(state, event, results, labels);
  }
  return state.items;
}
