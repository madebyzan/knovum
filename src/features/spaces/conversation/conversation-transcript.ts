import { z } from "zod";

import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { ConversationTaskUpdate } from "../contracts/generated/ConversationTaskUpdate";
import type { RunEvent } from "../contracts/generated/RunEvent";
import { toolCallIdOf } from "./conversation-activity-items";

/**
 * What a run said and did, in the order it happened: the words it wrote and the tools it called
 * between them, with the stored answers where the run's text was kept as messages.
 *
 * A turn is drawn from its transcript alone, whether the run is still producing or long settled.
 * The two read different sources — the live stream while it speaks, the ledger and the stored
 * messages afterwards — and both record in the order the Agent produced, so the transcript a
 * finished run reads back has the shape the live one had: nothing on screen moves when the run
 * settles, and a tool call never makes the words around it jump.
 */
export type TranscriptStep =
  | { kind: "text"; text: string }
  | { kind: "tool"; id: string }
  | { kind: "answer"; message: ConversationMessage };

/** The transcript of the run being streamed: its words and where its tool calls fell. */
export type LiveTranscript = readonly Extract<TranscriptStep, { kind: "text" | "tool" }>[];

function appendText(transcript: LiveTranscript, text: string): LiveTranscript {
  const last = transcript.at(-1);
  return last?.kind === "text"
    ? [...transcript.slice(0, -1), { kind: "text", text: last.text + text }]
    : [...transcript, { kind: "text", text }];
}

/** A call is placed where it was first reported; later reports about it change nothing here. */
function appendTool(transcript: LiveTranscript, id: string): LiveTranscript {
  return transcript.some((step) => step.kind === "tool" && step.id === id)
    ? transcript
    : [...transcript, { kind: "tool", id }];
}

/** The live transcript after one update from the stream. */
export function recordLiveUpdate(transcript: LiveTranscript, update: ConversationTaskUpdate): LiveTranscript {
  switch (update.type) {
    case "content_delta":
      return appendText(transcript, update.text);
    case "content_boundary": {
      // Another message begins. It is stored as its own answer, so it is its own block here.
      const last = transcript.at(-1);
      return last?.kind === "text" && last.text.trim()
        ? [...transcript, { kind: "text", text: "" }]
        : transcript;
    }
    case "tool_started":
      return appendTool(transcript, update.tool_call_id);
    case "finished":
      return transcript;
  }
}

const agentUpdateEvents = new Set<RunEvent["kind"]>([
  "external_agent_message",
  "external_agent_tool_started",
  "external_agent_tool_updated",
]);
const textPayload = z.object({ text: z.string() });

/**
 * A desktop Agent's transcript is its own updates, recorded in the ledger in arrival order. Its
 * stored answer holds the words after its last tool call, so the ledger's text stops there and
 * the answer takes over.
 */
function agentTranscript(events: readonly RunEvent[], answers: readonly ConversationMessage[]) {
  let steps: LiveTranscript = [];
  for (const event of events) {
    const text = event.kind === "external_agent_message" ? textPayload.safeParse(event.payload) : undefined;
    const toolCallId = toolCallIdOf(event);
    if (text?.success) steps = appendText(steps, text.data.text);
    else if (toolCallId !== undefined) steps = appendTool(steps, toolCallId);
  }
  const lastTool = steps.findLastIndex((step) => step.kind === "tool");
  return [
    ...steps.slice(0, lastTool + 1),
    ...answers.map((message) => ({ kind: "answer", message }) as const),
  ];
}

/**
 * The built-in Agent stores each message it writes as an answer, and each tool result as a tool
 * message after the answer that called it. A call with no result yet comes last.
 */
function storedTranscript(input: RecordedTranscriptInput, calledToolIds: Iterable<string>) {
  const answerIds = new Set(input.answers.map(({ id }) => id));
  const steps: TranscriptStep[] = [];
  const placed = new Set<string>();
  for (const message of input.messages) {
    if (answerIds.has(message.id)) steps.push({ kind: "answer", message });
    else if (message.role === "tool" && message.content.runId === input.runId)
      for (const part of message.content.parts)
        if (part.type === "tool_result" && !placed.has(part.callId)) {
          placed.add(part.callId);
          steps.push({ kind: "tool", id: part.callId });
        }
  }
  for (const id of calledToolIds) if (!placed.has(id)) steps.push({ kind: "tool", id });
  return steps;
}

interface RecordedTranscriptInput {
  runId: string;
  events: readonly RunEvent[];
  messages: readonly ConversationMessage[];
  answers: readonly ConversationMessage[];
}

/** The transcript of a run as it was recorded. */
export function recordedTranscript(
  input: RecordedTranscriptInput,
  calledToolIds: Iterable<string>,
): readonly TranscriptStep[] {
  return input.events.some((event) => agentUpdateEvents.has(event.kind))
    ? agentTranscript(input.events, input.answers)
    : storedTranscript(input, calledToolIds);
}
