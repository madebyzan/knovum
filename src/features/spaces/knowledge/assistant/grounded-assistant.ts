import { Channel, invoke } from "@tauri-apps/api/core";

import { loadAiExecutionTarget, aiExecutionTargetPayload } from "../../../../foundation/ai/ai-execution";
import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import type { GroundedMessage } from "../model/grounded-message";
import type { GroundedScope } from "../model/grounded-scope";
import type { KnowledgeSearchResponse } from "../model/knowledge-search-response";

export type GroundedAnswerStage = "retrieving" | "generating" | "saving";

export interface GroundedAnswer {
  threadId: string;
  user: GroundedMessage;
  assistant: GroundedMessage;
  semantic: KnowledgeSearchResponse["semantic"];
}

export interface AskGroundedKnowledgeInput {
  jobId: string;
  spaceId: string;
  scope: GroundedScope;
  nodeIds: readonly string[];
  question: string;
  threadId?: string;
  target: ExecutionTarget;
  onStage?: (stage: GroundedAnswerStage) => void;
  onDelta?: (text: string) => void;
}

type CompletionEvent =
  | { event: "started"; provider: string; model: string }
  | { event: "delta"; text: string };

export async function askGroundedKnowledge(input: AskGroundedKnowledgeInput): Promise<GroundedAnswer> {
  const target = await loadAiExecutionTarget("knowledge", input.target);
  const onEvent = new Channel<CompletionEvent>();
  // Tauri Channel exposes a single IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  onEvent.onmessage = (event) => {
    if (event.event === "delta") input.onDelta?.(event.text);
  };
  const onStage = new Channel<GroundedAnswerStage>();
  // Tauri Channel exposes a single IPC receiver rather than a DOM EventTarget.
  // oxlint-disable-next-line unicorn/prefer-add-event-listener
  onStage.onmessage = (stage) => input.onStage?.(stage);
  return invoke("plugin:spaces|answer_grounded_question", {
    request: {
      jobId: input.jobId,
      spaceId: input.spaceId,
      scope: input.scope,
      nodeIds: [...input.nodeIds],
      question: input.question,
      threadId: input.threadId ?? null,
      target: aiExecutionTargetPayload(target),
    },
    onEvent,
    onStage,
  });
}
