import { normalizeError } from "../../../foundation/errors/app-error";
import type { ConversationTaskAccepted } from "../contracts/generated/ConversationTaskAccepted";
import type { ConversationTaskUpdate } from "../contracts/generated/ConversationTaskUpdate";
import { getConversationTaskError } from "./conversation-ipc";

type FinishedUpdate = Extract<ConversationTaskUpdate, { type: "finished" }>;

interface ConversationTaskSettlementInput {
  accepted: ConversationTaskAccepted;
  outcome: FinishedUpdate;
  spaceId: string;
  project: (task: ConversationTaskAccepted, spaceId: string) => Promise<void>;
}

const successfulStates = new Set(["succeeded", "paused", "waiting", "cancelled"]);

export async function settleConversationTaskResult(input: ConversationTaskSettlementInput) {
  let projectionError: Error | undefined;
  try {
    await input.project(input.accepted, input.spaceId);
  } catch (reason) {
    projectionError = normalizeError(reason);
  }
  if (input.outcome.error) {
    const terminalError = normalizeError(input.outcome.error);
    if (projectionError)
      throw new AggregateError([terminalError, projectionError], terminalError.message, {
        cause: terminalError,
      });
    throw terminalError;
  }
  if (successfulStates.has(input.outcome.state)) {
    if (projectionError) throw projectionError;
    return;
  }
  let reason: string | null;
  try {
    reason = await getConversationTaskError(input.accepted.runId);
  } catch (error) {
    const diagnosticError = normalizeError(error);
    if (projectionError)
      throw new AggregateError([diagnosticError, projectionError], diagnosticError.message, {
        cause: diagnosticError,
      });
    throw diagnosticError;
  }
  const terminalError = new Error(reason ?? `Conversation task ended as ${input.outcome.state}`);
  if (projectionError)
    throw new AggregateError([terminalError, projectionError], terminalError.message, {
      cause: terminalError,
    });
  throw terminalError;
}
