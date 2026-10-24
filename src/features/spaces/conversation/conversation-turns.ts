import type { ConversationMessage } from "../contracts/generated/ConversationMessage";

/**
 * One exchange: what a person asked, and everything that answered it.
 *
 * The list renders turns rather than messages because every part of an answer — the tools the run
 * used, the text it wrote, the nodes it added, the sources it cited — belongs to the question that
 * caused it. Grouping it here is also what keeps the answer in flight inside the list: a turn
 * carries the text that is still arriving, so nothing has to be rendered beside the list and then
 * moved into it when the message lands.
 */
export interface ConversationTurn {
  id: string;
  /** Absent for an answer with no question before it, which a branch can leave behind. */
  prompt: ConversationMessage | undefined;
  /** The run that answered, once the ledger has one; absent for a question just sent. */
  runId: string | undefined;
  answers: readonly ConversationMessage[];
  /** The answer as it arrives, replacing the stored text of the last answer until it lands. */
  streamedText: string;
  /** The run answering this turn is live. */
  active: boolean;
  /**
   * Being worked on with nothing written yet, and which of the two waits it is: `preparing` while
   * the question has no run yet — the models it needs may still be loading — and `generating` once
   * a run is producing the answer.
   *
   * It is the opening state of a turn and does not come back, because the live text keeps every
   * block the run has written: once there are words there is something to show until the stored
   * answer replaces them.
   */
  pending: "preparing" | "generating" | undefined;
}

export interface ConversationTurnsInput {
  messages: readonly ConversationMessage[];
  activeRunId: string | undefined;
  streamedText: string;
  /** A question was sent and no run has been accepted for it yet. */
  preparing: boolean;
}

/** Whether an answer has anything to put on screen at all. */
function answerShows(message: ConversationMessage): boolean {
  const { parts, outputs, citations, status } = message.content;
  return (
    parts.some((part) => part.type !== "text" || part.text.trim()) ||
    Boolean(outputs?.length) ||
    citations.length > 0 ||
    Boolean(status)
  );
}

export function conversationTurns({
  messages,
  activeRunId,
  streamedText,
  preparing,
}: ConversationTurnsInput): ConversationTurn[] {
  const turns = groupTurns(messages);
  const last = turns.at(-1);
  // Only the newest turn can be in flight: a question sent before its run exists has no run id to
  // match, and once it has one the ledger keeps it.
  const working = Boolean(activeRunId) || preparing;
  if (!last || !working || (last.runId !== undefined && last.runId !== activeRunId)) return turns;
  last.active = true;
  last.streamedText = streamedText;
  if (!streamedText && last.answers.length === 0) last.pending = activeRunId ? "generating" : "preparing";
  return turns;
}

function groupTurns(messages: readonly ConversationMessage[]): ConversationTurn[] {
  const turns: ConversationTurn[] = [];
  for (const message of messages) {
    // Tool messages are the transcript the activity list is built from, not a side of the exchange.
    if (message.role === "tool") continue;
    const open = message.role === "assistant" ? turns.at(-1) : undefined;
    if (open) {
      if (answerShows(message)) open.answers = [...open.answers, message];
      open.runId ??= message.content.runId ?? undefined;
      continue;
    }
    turns.push(openTurn(message));
  }
  return turns;
}

function openTurn(message: ConversationMessage): ConversationTurn {
  return {
    id: message.id,
    prompt: message.role === "user" ? message : undefined,
    runId: message.content.runId ?? undefined,
    answers: message.role === "assistant" && answerShows(message) ? [message] : [],
    streamedText: "",
    active: false,
    pending: undefined,
  };
}
