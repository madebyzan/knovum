import { describe, expect, it } from "vitest";

import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import type { RunEvent } from "../../contracts/generated/RunEvent";
import { conversationActivityItems } from "../conversation-activity-items";

function event(kind: RunEvent["kind"], payload: RunEvent["payload"], sequence: number): RunEvent {
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

const labels = { action: (actionId: string) => actionId, default: "Tool" };

describe("conversation tool activity", () => {
  it("retains input context when completion only updates status and attaches the tool output", () => {
    const messages: ConversationMessage[] = [
      {
        id: "message",
        conversationId: "conversation",
        branchId: "branch",
        role: "tool",
        createdAt: 3,
        content: {
          runId: "run",
          citations: [],
          parts: [{ type: "tool_result", callId: "call", name: "search", result: { matches: [] } }],
        },
      },
    ];
    const items = conversationActivityItems({
      runId: "run",
      labels,
      messages,
      events: [
        event(
          "conversation_tool_started",
          { toolCallId: "call", actionId: "search", context: { query: "audio" } },
          1,
        ),
        event("conversation_tool_finished", { toolCallId: "call" }, 2),
      ],
    });
    expect(items).toEqual([
      {
        id: "call",
        title: "search",
        status: "completed",
        context: { query: "audio" },
        output: { matches: [] },
        outputName: "search",
      },
    ]);
  });

  it("projects external tool updates using the owned context field", () => {
    const items = conversationActivityItems({
      runId: "run",
      labels,
      messages: [],
      events: [
        event(
          "external_agent_tool_started",
          { toolCallId: "call", title: "Read file", status: "pending", context: "/audio.wav" },
          1,
        ),
        event(
          "external_agent_tool_updated",
          { toolCallId: "call", status: "completed", context: "Read 128 bytes" },
          2,
        ),
      ],
    });
    expect(items).toEqual([
      { id: "call", title: "Read file", status: "completed", context: "Read 128 bytes", output: undefined },
    ]);
  });

  it("attaches MCP progress to the tool it reports on and keeps it across later updates", () => {
    const items = conversationActivityItems({
      runId: "run",
      labels,
      messages: [],
      events: [
        event("conversation_tool_started", { toolCallId: "call", title: "transcribe" }, 1),
        event("mcp_progress", { toolCallId: "call", progress: 2, total: 5, message: null }, 2),
        event("mcp_progress", { toolCallId: "unknown", progress: 9 }, 3),
        event("external_agent_tool_updated", { toolCallId: "call", status: "in_progress" }, 4),
      ],
    });
    expect(items).toEqual([
      {
        id: "call",
        title: "transcribe",
        status: "in_progress",
        context: undefined,
        output: undefined,
        progress: { progress: 2, total: 5, message: null },
      },
    ]);
  });

  it("rejects obsolete event fields instead of reading a second payload vocabulary", () => {
    expect(
      conversationActivityItems({
        runId: "run",
        labels,
        messages: [],
        events: [
          event(
            "conversation_tool_started",
            { toolCallId: "call", actionId: "search", detail: { query: "audio" } },
            1,
          ),
        ],
      }),
    ).toEqual([]);
  });
});
