import { describe, expect, it } from "vitest";

import type { ConversationMessage } from "../../contracts/generated/ConversationMessage";
import type { ConversationMessageRole } from "../../contracts/generated/ConversationMessageRole";
import { conversationTurns } from "../conversation-turns";

function message(
  id: string,
  role: ConversationMessageRole,
  text: string,
  runId?: string,
): ConversationMessage {
  return {
    id,
    conversationId: "conversation",
    branchId: "branch",
    role,
    createdAt: 1,
    content: {
      parts: text ? [{ type: "text", text }] : [],
      citations: [],
      ...(runId ? { runId } : {}),
    },
  };
}

describe("conversation turns", () => {
  it("groups an answer with the question that caused it", () => {
    const turns = conversationTurns({
      messages: [
        message("ask-1", "user", "First question", "run-1"),
        message("tool-1", "tool", "tool transcript", "run-1"),
        message("say-1", "assistant", "First answer", "run-1"),
        message("ask-2", "user", "Second question", "run-2"),
      ],
      activeRunId: undefined,
      streamedText: "",
      preparing: false,
    });

    expect(turns.map((turn) => turn.id)).toEqual(["ask-1", "ask-2"]);
    expect(turns[0]?.answers.map((answer) => answer.id)).toEqual(["say-1"]);
    // Tool messages are the transcript the activity list is built from, not a side of the exchange.
    expect(turns[0]?.runId).toBe("run-1");
    expect(turns[1]?.answers).toEqual([]);
  });

  it("drops an answer that has nothing to show, and keeps one that only produced a node", () => {
    const outputs = message("say-empty", "assistant", "", "run-1");
    const withOutput = {
      ...outputs,
      id: "say-output",
      content: { ...outputs.content, outputs: [{ nodeId: "node", title: "Chart", kind: "image" }] },
    } as ConversationMessage;

    const turns = conversationTurns({
      messages: [message("ask-1", "user", "Draw it", "run-1"), outputs, withOutput],
      activeRunId: undefined,
      streamedText: "",
      preparing: false,
    });

    expect(turns[0]?.answers.map((answer) => answer.id)).toEqual(["say-output"]);
  });

  it("carries the answer in flight on the question that is still waiting for it", () => {
    const sent = [message("ask-1", "user", "Explain this")];

    // A question sent before its run exists has no run id to match; the turn is still the one in
    // flight, and says so itself instead of a notice floating at the end of the list.
    const preparing = conversationTurns({
      messages: sent,
      activeRunId: undefined,
      streamedText: "",
      preparing: true,
    });
    expect(preparing[0]).toMatchObject({ active: true, pending: "preparing", streamedText: "" });

    // The first words arrive before the message that will hold them exists, and they belong to the
    // same turn — not to a block rendered beside the list and moved into it later.
    const streaming = conversationTurns({
      messages: sent,
      activeRunId: "run-1",
      streamedText: "Because",
      preparing: false,
    });
    expect(streaming[0]).toMatchObject({
      runId: "run-1",
      active: true,
      pending: undefined,
      streamedText: "Because",
    });

    // When a run is paused or waiting on user approval, it owns the turn and any partial text,
    // but must not claim to be actively generating.
    const paused = conversationTurns({
      messages: sent,
      activeRunId: "run-1",
      streamedText: "",
      preparing: false,
      producing: false,
    });
    expect(paused[0]).toMatchObject({
      runId: "run-1",
      active: false,
      pending: undefined,
      streamedText: "",
    });
  });

  it("leaves a settled exchange alone while another conversation runs", () => {
    const turns = conversationTurns({
      messages: [
        message("ask-1", "user", "Explain this", "run-1"),
        message("say-1", "assistant", "Answer", "run-1"),
      ],
      activeRunId: "run-elsewhere",
      streamedText: "",
      preparing: false,
    });

    expect(turns[0]).toMatchObject({ active: false, pending: undefined, streamedText: "" });
  });
});
