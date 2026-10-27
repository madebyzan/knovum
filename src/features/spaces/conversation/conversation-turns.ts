import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { LiveTranscript } from "./conversation-transcript";

/**
 * One exchange: what a person asked, and everything that answered it.
 *
 * The list renders turns rather than messages because every part of an answer — the tools the run
 * used, the text it wrote, the nodes it added, the sources it cited — belongs to the question that
 * caused it. Grouping it here is also what keeps the answer in flight inside the list: a turn
 * carries the transcript that is still arriving, so nothing has to be rendered beside the list and
 * then moved into it when the message lands.
 */
export interface ConversationTurn {
  id: string;
  /** Absent for an answer with no question before it, which a branch can leave behind. */
  prompt: ConversationMessage | undefined;
  /** The run that answered, once the ledger has one; absent for a question just sent. */
  runId: string | undefined;
  answers: readonly ConversationMessage[];
  /**
   * What the run has said and done so far, from the stream that is carrying it. Present only for
   * the turn this window is streaming; any other turn — and this one once it settles — is read
   * from what the run recorded.
   */
  live: LiveTranscript | undefined;
  /** The run answering this turn is producing, rather than paused or waiting on someone. */
  active: boolean;
}

export interface ConversationTurnsInput {
  messages: readonly ConversationMessage[];
  activeRunId: string | undefined;
  live: LiveTranscript | undefined;
  /** A question was sent and no run has been accepted for it yet. */
  preparing: boolean;
  /** Whether the active run is currently producing rather than paused or waiting on user action. */
  producing?: boolean | undefined;
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

function isInFlightTurn(
  turn: ConversationTurn | undefined,
  activeRunId: string | undefined,
  preparing: boolean,
): turn is ConversationTurn {
  if (!turn || (!activeRunId && !preparing)) return false;
  return turn.runId === undefined || turn.runId === activeRunId;
}

export function conversationTurns({
  messages,
  activeRunId,
  live,
  preparing,
  producing = true,
}: ConversationTurnsInput): ConversationTurn[] {
  const turns = groupTurns(messages);
  const last = turns.at(-1);
  // Only the newest turn can be in flight: a question sent before its run exists has no run id to
  // match, and once it has one the ledger keeps it.
  if (!isInFlightTurn(last, activeRunId, preparing)) return turns;
  last.runId ??= activeRunId;
  last.live = live;
  last.active = producing;
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
    live: undefined,
    active: false,
  };
}
