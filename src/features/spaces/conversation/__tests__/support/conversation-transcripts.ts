import type { ConversationMessage } from "../../../contracts/generated/ConversationMessage";
import type { ConversationMessageRole } from "../../../contracts/generated/ConversationMessageRole";
import type { RunEvent } from "../../../contracts/generated/RunEvent";

export function runEvent(kind: RunEvent["kind"], payload: RunEvent["payload"], sequence: number): RunEvent {
  return {
    sequence,
    runId: "run",
    stepInstanceId: null,
    attemptId: null,
    kind,
    payload,
    createdAt: sequence,
  };
}

export function conversationMessage(
  id: string,
  role: ConversationMessageRole,
  text: string,
  runId: string | undefined = "run",
): ConversationMessage {
  return {
    id,
    conversationId: "conversation",
    branchId: "branch",
    role,
    createdAt: 1,
    content: { parts: text ? [{ type: "text", text }] : [], citations: [], ...(runId ? { runId } : {}) },
  };
}

export function toolResult(id: string, callId: string): ConversationMessage {
  return {
    ...conversationMessage(id, "tool", ""),
    content: {
      runId: "run",
      citations: [],
      parts: [{ type: "tool_result", callId, name: "search", result: {} }],
    },
  };
}

/**
 * A desktop Agent that says what it will do, calls two tools, and answers — as its ledger and
 * stored answer keep it once settled.
 */
export function recordedAgentRun() {
  const events = [
    runEvent("external_agent_message", { text: "I'll look " }, 1),
    runEvent("external_agent_message", { text: "it up." }, 2),
    runEvent(
      "external_agent_tool_started",
      { toolCallId: "search", title: "search_knowledge", status: "pending" },
      3,
    ),
    runEvent("external_agent_tool_updated", { toolCallId: "search", status: "completed" }, 4),
    runEvent("external_agent_tool_started", { toolCallId: "node", title: "get_node", status: "pending" }, 5),
    runEvent("external_agent_tool_updated", { toolCallId: "node", status: "completed" }, 6),
    runEvent("external_agent_message", { text: "Found three notes." }, 7),
  ];
  const answer = conversationMessage("answer", "assistant", "Found three notes.");
  return { events, answer };
}

/**
 * The built-in Agent's run: a message that calls a tool, then the answer, each stored as it was
 * written, with the tool's result between them.
 */
export function recordedBuiltinRun() {
  const first = conversationMessage("first", "assistant", "Checking the notes.");
  const answer = conversationMessage("answer", "assistant", "They agree.");
  const events = [
    runEvent("conversation_tool_started", { toolCallId: "search", title: "search" }, 1),
    runEvent("conversation_tool_finished", { toolCallId: "search", status: "completed" }, 2),
  ];
  return { events, messages: [first, toolResult("result", "search"), answer], answers: [first, answer] };
}
