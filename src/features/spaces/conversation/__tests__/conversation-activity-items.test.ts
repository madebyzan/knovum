import { describe, expect, it } from "vitest";

import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import { conversationToolItems } from "../conversation-activity-items";
import { runEvent } from "./support/conversation-transcripts";

const labels = { action: (actionId: string) => actionId, default: "Tool" };

function items(
  events: Parameters<typeof conversationToolItems>[0]["events"],
  messages: ConversationMessage[] = [],
) {
  return [...conversationToolItems({ runId: "run", labels, messages, events }).values()];
}

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
    expect(
      items(
        [
          runEvent(
            "conversation_tool_started",
            { toolCallId: "call", actionId: "search", context: { query: "audio" } },
            1,
          ),
          runEvent("conversation_tool_finished", { toolCallId: "call" }, 2),
        ],
        messages,
      ),
    ).toEqual([
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
    expect(
      items([
        runEvent(
          "external_agent_tool_started",
          { toolCallId: "call", title: "Read file", status: "pending", context: "/audio.wav" },
          1,
        ),
        runEvent(
          "external_agent_tool_updated",
          { toolCallId: "call", status: "completed", context: "Read 128 bytes" },
          2,
        ),
      ]),
    ).toEqual([{ id: "call", title: "Read file", status: "completed", context: "Read 128 bytes" }]);
  });

  it("attaches MCP progress to the tool it reports on and keeps it across later updates", () => {
    expect(
      items([
        runEvent("conversation_tool_started", { toolCallId: "call", title: "transcribe" }, 1),
        runEvent("mcp_progress", { toolCallId: "call", progress: 2, total: 5, message: null }, 2),
        runEvent("mcp_progress", { toolCallId: "unknown", progress: 9 }, 3),
        runEvent("external_agent_tool_updated", { toolCallId: "call", status: "in_progress" }, 4),
      ]),
    ).toEqual([
      {
        id: "call",
        title: "transcribe",
        status: "in_progress",
        context: undefined,
        progress: { progress: 2, total: 5, message: null },
      },
    ]);
  });

  it("describes calls only: what the run said is the transcript's", () => {
    expect(
      items([
        runEvent("external_agent_message", { text: "Searching the Space." }, 1),
        runEvent("conversation_tool_started", { toolCallId: "call", title: "search_knowledge" }, 2),
      ]).map((item) => item.title),
    ).toEqual(["search_knowledge"]);
  });

  it("rejects obsolete event fields instead of reading a second payload vocabulary", () => {
    expect(
      items([
        runEvent(
          "conversation_tool_started",
          { toolCallId: "call", actionId: "search", detail: { query: "audio" } },
          1,
        ),
      ]),
    ).toEqual([]);
  });
});
