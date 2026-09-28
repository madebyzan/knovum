import { z } from "zod";

import type { RunEvent } from "../contracts/generated/RunEvent";
import type { AssistantMessage } from "./conversation-ipc";

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
const permissionPayload = z.object({
  requestId: z.string(),
  title: z.string(),
  options: z.array(z.object({ label: z.string() })),
});
const toolEvents = new Set([
  "external_agent_tool_started", "external_agent_tool_updated",
  "assistant_tool_started", "assistant_tool_finished",
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

function appendCommentary(state: ActivityState, event: RunEvent) {
  if (!state.commentary.trim()) return;
  state.items.push({ id: `text:${event.sequence}`, title: state.commentary.trim(), status: "commentary" });
  state.commentary = "";
}

function appendPermission(state: ActivityState, event: RunEvent) {
  const parsed = permissionPayload.safeParse(event.payload);
  if (!parsed.success) return;
  appendCommentary(state, event);
  state.items.push({
    id: `permission:${parsed.data.requestId}`,
    title: parsed.data.title,
    status: "permission",
    detail: parsed.data.options.map((option) => option.label).join(" · "),
  });
}

function appendTool(
  state: ActivityState,
  event: RunEvent,
  results: Map<string, { name: string; value: unknown }>,
  actionLabel: (id: string) => string,
  defaultLabel: string,
) {
  if (!toolEvents.has(event.kind)) return;
  const parsed = toolPayload.safeParse(event.payload);
  if (!parsed.success) return;
  appendCommentary(state, event);
  const { toolCallId, title, actionId, status, detail } = parsed.data;
  const index = state.positions.get(toolCallId);
  const previous = index === undefined ? undefined : state.items[index];
  const item: ActivityItem = {
    id: toolCallId,
    title: title ?? previous?.title ?? (actionId ? actionLabel(actionId) : defaultLabel),
    status: status ?? (event.kind === "assistant_tool_finished" ? "completed" : previous?.status ?? "in_progress"),
    detail: detail == null ? previous?.detail : detail,
    result: results.get(toolCallId)?.value,
    resultName: results.get(toolCallId)?.name,
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
  actionLabel: (id: string) => string,
  defaultLabel: string,
) {
  const results = toolResults(messages, runId);
  const state: ActivityState = { items: [], positions: new Map(), commentary: "" };
  for (const event of events) {
    if (event.kind === "external_agent_message") {
      const parsed = textPayload.safeParse(event.payload);
      if (parsed.success) state.commentary += parsed.data.text;
    } else if (event.kind === "external_agent_permission_requested") appendPermission(state, event);
    else appendTool(state, event, results, actionLabel, defaultLabel);
  }
  return state.items;
}
