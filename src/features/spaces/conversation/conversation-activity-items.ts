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

type ActivityState = {
  items: ActivityItem[];
  indexByCallId: Map<string, number>;
  progressByCallId: Map<string, ToolProgress>;
  commentary: string;
  live: boolean;
};
type ToolLabels = { action: (actionId: string) => string; default: string };

function appendCommentary(state: ActivityState, event: RunEvent) {
  const text = state.commentary.trim();
  state.commentary = "";
  if (!text || state.live) return;
  state.items.push({ id: `text:${event.sequence}`, title: text, status: "commentary" });
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

function recordProgress(state: ActivityState, event: RunEvent) {
  const parsed = progressPayload.safeParse(event.payload);
  if (!parsed.success) return;
  const { toolCallId, ...progress } = parsed.data;
  state.progressByCallId.set(toolCallId, progress);
}

interface ConversationActivityInput {
  events: readonly RunEvent[];
  messages: readonly ConversationMessage[];
  runId: string;
  labels: ToolLabels;
  /**
   * Whether the run is still producing. While it is, the words it has spoken are on screen from
   * the live stream, which has them first and in full; the ledger is recording the same words and
   * would show them a second time, a refresh behind. A finished run is read from the ledger alone,
   * where the commentary is back in place between the tools it explains.
   */
  live: boolean;
}

export function conversationActivityItems({
  events,
  messages,
  runId,
  labels,
  live,
}: ConversationActivityInput) {
  const outputByCallId = toolOutputs(messages, runId);
  const state: ActivityState = {
    items: [],
    indexByCallId: new Map(),
    progressByCallId: new Map(),
    commentary: "",
    live,
  };
  for (const event of events) {
    if (event.kind === "external_agent_message") {
      const parsed = textPayload.safeParse(event.payload);
      if (parsed.success) state.commentary += parsed.data.text;
    } else if (event.kind === "mcp_progress") recordProgress(state, event);
    else appendTool(state, event, outputByCallId, labels);
  }
  for (const item of state.items) {
    const progress = state.progressByCallId.get(item.id);
    if (progress) item.progress = progress;
  }
  return state.items;
}
