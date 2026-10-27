import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import type { ActivityItem } from "./conversation-activity-items";
import type { TranscriptStep } from "./conversation-transcript";

/**
 * What a turn shows, top to bottom: the words the run wrote along the way, the tools it called
 * between them — consecutive calls folded into one group — and the stored answers.
 */
export type TimelineBlock =
  | { kind: "text"; key: string; text: string }
  | { kind: "tools"; key: string; items: readonly ActivityItem[] }
  | { kind: "answer"; key: string; message: ConversationMessage; final: boolean };

type ToolGroup = { kind: "tools"; key: string; ids: string[] };

/**
 * Keys follow the transcript, not the screen: the n-th block of words and the n-th group of
 * calls keep their keys while the calls before them are still being looked up. A block therefore
 * keeps its element — and a group its open or closed state — as the run goes on and when it
 * settles. A group is not named by its calls: calls made together reach the stream in whatever
 * order they started, and a finished run reads them back in the order their results were stored.
 */
export function conversationTimeline(
  steps: readonly TranscriptStep[],
  tools: ReadonlyMap<string, ActivityItem>,
): TimelineBlock[] {
  const blocks: (Exclude<TimelineBlock, { kind: "tools" }> | ToolGroup)[] = [];
  let texts = 0;
  let groups = 0;
  for (const step of steps) {
    const last = blocks.at(-1);
    if (step.kind === "tool") {
      if (last?.kind === "tools") last.ids.push(step.id);
      else blocks.push({ kind: "tools", key: `tools:${groups++}`, ids: [step.id] });
    } else if (step.kind === "answer") {
      blocks.push({ kind: "answer", key: step.message.id, message: step.message, final: false });
    } else if (step.text.trim()) {
      blocks.push({ kind: "text", key: `text:${texts++}`, text: step.text });
    }
  }
  const finalAnswer = blocks.findLast((block) => block.kind === "answer");
  if (finalAnswer) finalAnswer.final = true;
  return blocks.flatMap((block) => (block.kind === "tools" ? toolBlock(block, tools) : [block]));
}

/**
 * A group lists its calls in the order the ledger saw them, which is the order a finished run
 * reads back; a group whose calls are not in the ledger yet has nothing to show.
 */
function toolBlock({ key, ids }: ToolGroup, tools: ReadonlyMap<string, ActivityItem>): TimelineBlock[] {
  const members = new Set(ids);
  const items = [...tools.values()].filter((item) => members.has(item.id));
  return items.length ? [{ kind: "tools", key, items }] : [];
}
