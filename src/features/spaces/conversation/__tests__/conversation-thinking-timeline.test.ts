import { describe, expect, it } from "vitest";

import { conversationToolItems } from "../conversation-activity-items";
import { conversationTimeline } from "../conversation-timeline";
import { recordLiveUpdate, recordedTranscript, type LiveTranscript } from "../conversation-transcript";
import {
  recordedAgentRun,
  recordedBuiltinRun,
  runEvent,
  toolResult,
} from "./support/conversation-transcripts";

const labels = { action: (actionId: string) => actionId, default: "Tool" };

function stream(...updates: Parameters<typeof recordLiveUpdate>[1][]): LiveTranscript {
  return updates.reduce(recordLiveUpdate, [] as LiveTranscript);
}

function thinking(text: string) {
  return { type: "thinking_delta", task_id: "task", text } as const;
}

function delta(text: string) {
  return { type: "content_delta", task_id: "task", text } as const;
}

function tool(id: string) {
  return { type: "tool_started", tool_call_id: id } as const;
}

function contents(blocks: ReturnType<typeof conversationTimeline>) {
  return blocks.map((block) => {
    if (block.kind === "tools") return block.items.map((item) => item.id);
    if (block.kind === "thinking") return `thought:${block.text}`;
    if (block.kind === "answer")
      return block.message.content.parts.map((part) => (part.type === "text" ? part.text : "")).join("");
    return block.text;
  });
}

describe("conversation thinking timeline", () => {
  it("matches live and settled thinking order for desktop Agent runs including post-tool reasoning", () => {
    const { events: baseEvents, answer } = recordedAgentRun();
    const middle = baseEvents
      .slice(0, 6)
      .map((event, index) => Object.assign({}, event, { sequence: index + 1, createdAt: 2000 + index }));
    const events = [
      Object.assign(runEvent("agent_thinking", { text: "First " }, 0), { createdAt: 1000 }),
      Object.assign(runEvent("agent_thinking", { text: "thought." }, 0), { createdAt: 1400 }),
      ...middle,
      Object.assign(runEvent("agent_thinking", { text: "Synthesizing results." }, 8), { createdAt: 3000 }),
      Object.assign({}, baseEvents[6]!, { sequence: 9, createdAt: 4500 }),
    ];
    const tools = conversationToolItems({ events, messages: [], runId: "run", labels });
    const live = stream(
      thinking("First "),
      thinking("thought."),
      delta("I'll look it up."),
      tool("search"),
      tool("node"),
      thinking("Synthesizing results."),
      delta("Found three notes."),
    );
    const settled = recordedTranscript(
      { runId: "run", events, messages: [answer], answers: [answer] },
      tools.keys(),
    );

    const liveBlocks = conversationTimeline(live, tools);
    const settledBlocks = conversationTimeline(settled, tools);
    expect(contents(settledBlocks)).toEqual(contents(liveBlocks));
    expect(settledBlocks[0]).toMatchObject({ kind: "thinking", key: "thinking:0", durationMs: 1000 });
    expect(settledBlocks[3]).toMatchObject({ kind: "thinking", key: "thinking:1", durationMs: 1500 });
  });

  it("interleaves built-in thinking ahead of parallel out-of-order tools and final answers", () => {
    const { messages, answers } = recordedBuiltinRun();
    const [first, , answer] = messages;
    const events = [
      runEvent("agent_thinking", { text: "Need two tools.", durationMs: 1200 }, 1),
      runEvent("conversation_tool_started", { toolCallId: "search", title: "search" }, 2),
      runEvent("conversation_tool_started", { toolCallId: "node", title: "node" }, 3),
      runEvent("conversation_tool_finished", { toolCallId: "node", status: "completed" }, 4),
      runEvent("conversation_tool_finished", { toolCallId: "search", status: "completed" }, 5),
      runEvent("agent_thinking", { text: "Ready to summarize.", durationMs: 800 }, 6),
    ];
    const stored = [
      first!,
      toolResult("node-result", "node"),
      toolResult("search-result", "search"),
      answer!,
    ];
    const tools = conversationToolItems({ events, messages: stored, runId: "run", labels });
    const live = stream(
      thinking("Need two tools."),
      delta("Checking the notes."),
      tool("search"),
      tool("node"),
      { type: "content_boundary" },
      thinking("Ready to summarize."),
      delta("They agree."),
    );
    const settled = recordedTranscript({ runId: "run", events, messages: stored, answers }, tools.keys());

    expect(contents(conversationTimeline(settled, tools))).toEqual(
      contents(conversationTimeline(live, tools)),
    );
  });
});
