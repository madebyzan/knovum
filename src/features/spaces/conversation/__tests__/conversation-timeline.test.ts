import { describe, expect, it } from "vitest";

import { conversationToolItems } from "../conversation-activity-items";
import { conversationTimeline } from "../conversation-timeline";
import { recordLiveUpdate, recordedTranscript, type LiveTranscript } from "../conversation-transcript";
import { recordedAgentRun, recordedBuiltinRun } from "./support/conversation-transcripts";

const labels = { action: (actionId: string) => actionId, default: "Tool" };

function stream(...updates: Parameters<typeof recordLiveUpdate>[1][]): LiveTranscript {
  return updates.reduce(recordLiveUpdate, [] as LiveTranscript);
}

function delta(text: string) {
  return { type: "content_delta", task_id: "task", text } as const;
}

function tool(id: string) {
  return { type: "tool_started", tool_call_id: id } as const;
}

/** What the screen shows, without the elements that render it: the content of each block. */
function contents(blocks: ReturnType<typeof conversationTimeline>) {
  return shape(blocks).map((block) => ("tools" in block ? block.tools : block.text));
}

/** Each block's content, and the key that keeps its element across updates. */
function shape(blocks: ReturnType<typeof conversationTimeline>) {
  return blocks.map((block) =>
    block.kind === "tools"
      ? { tools: block.items.map((item) => item.id), key: block.key }
      : block.kind === "answer"
        ? {
            text: block.message.content.parts.map((part) => (part.type === "text" ? part.text : "")).join(""),
          }
        : { text: block.text, key: block.key },
  );
}

describe("conversation timeline", () => {
  it("puts a desktop Agent's tool calls between the words around them", () => {
    const { events, answer } = recordedAgentRun();
    const tools = conversationToolItems({ events, messages: [], runId: "run", labels });
    const blocks = conversationTimeline(
      recordedTranscript({ runId: "run", events, messages: [answer], answers: [answer] }, tools.keys()),
      tools,
    );

    expect(shape(blocks)).toEqual([
      { text: "I'll look it up.", key: "text:0" },
      { tools: ["search", "node"], key: "tools:search" },
      { text: "Found three notes." },
    ]);
    expect(blocks.at(-1)).toMatchObject({ kind: "answer", final: true });
  });

  it("reads a settled desktop Agent run with the shape it had while streaming", () => {
    // The live stream and the ledger record the same updates in the same order. Were their
    // shapes to differ, the turn would rearrange itself the moment the run settles.
    const { events, answer } = recordedAgentRun();
    const tools = conversationToolItems({ events, messages: [], runId: "run", labels });
    const live = stream(
      delta("I'll look "),
      delta("it up."),
      tool("search"),
      tool("node"),
      delta("Found three notes."),
    );
    const settled = recordedTranscript(
      { runId: "run", events, messages: [answer], answers: [answer] },
      tools.keys(),
    );

    expect(contents(conversationTimeline(live, tools))).toEqual(
      contents(conversationTimeline(settled, tools)),
    );
  });

  it("reads a settled built-in Agent run with the shape it had while streaming", () => {
    const { events, messages, answers } = recordedBuiltinRun();
    const tools = conversationToolItems({ events, messages, runId: "run", labels });
    const live = stream(
      delta("Checking the notes."),
      tool("search"),
      { type: "content_boundary" },
      delta("They agree."),
    );
    const settled = recordedTranscript({ runId: "run", events, messages, answers }, tools.keys());

    expect(contents(conversationTimeline(live, tools))).toEqual([
      "Checking the notes.",
      ["search"],
      "They agree.",
    ]);
    expect(contents(conversationTimeline(settled, tools))).toEqual(
      contents(conversationTimeline(live, tools)),
    );
    // Only the answer that ends the turn offers its actions; the words before it are part of it.
    expect(
      conversationTimeline(settled, tools).map((block) => block.kind === "answer" && block.final),
    ).toEqual([false, false, true]);
  });

  it("keeps a tool group's key and the words after it while the ledger has not described the call yet", () => {
    const live = stream(delta("Looking."), tool("search"), delta("Done."));
    const before = conversationTimeline(live, new Map());
    const { events } = recordedAgentRun();
    const after = conversationTimeline(
      live,
      conversationToolItems({ events, messages: [], runId: "run", labels }),
    );

    expect(shape(before)).toEqual([
      { text: "Looking.", key: "text:0" },
      { text: "Done.", key: "text:1" },
    ]);
    expect(shape(after)).toEqual([
      { text: "Looking.", key: "text:0" },
      { tools: ["search"], key: "tools:search" },
      { text: "Done.", key: "text:1" },
    ]);
  });

  it("places a built-in call with no result yet after everything recorded", () => {
    const { events, messages, answers } = recordedBuiltinRun();
    const pending = [
      ...events,
      { ...events[0]!, sequence: 3, payload: { toolCallId: "late", title: "read" } },
    ];
    const tools = conversationToolItems({ events: pending, messages, runId: "run", labels });
    const steps = recordedTranscript({ runId: "run", events: pending, messages, answers }, tools.keys());

    expect(
      steps.map((step) =>
        step.kind === "answer" ? step.message.id : step.kind === "tool" ? step.id : step.text,
      ),
    ).toEqual(["first", "search", "answer", "late"]);
  });
});
