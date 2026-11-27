import { z } from "zod";

import type { RunEvent } from "../contracts/generated/RunEvent";

export interface ThinkingStep {
  kind: "thinking";
  text: string;
  startedAt?: number | undefined;
  durationMs?: number | undefined;
}

const thinkingPayload = z.object({ text: z.string(), durationMs: z.number().optional() });
const startedToolPayload = z.object({ toolCallId: z.string() });

function isThinkingStep(step: { kind: string } | undefined): step is ThinkingStep {
  return step?.kind === "thinking";
}

export function parseThinkingEvent(
  event: RunEvent,
): { text: string; durationMs?: number | undefined } | undefined {
  if (event.kind !== "agent_thinking") return undefined;
  const parsed = thinkingPayload.safeParse(event.payload);
  return parsed.success && parsed.data.text ? parsed.data : undefined;
}

export function appendLiveThinking<T extends { kind: string }>(
  transcript: readonly (T | ThinkingStep)[],
  text: string,
  startedAt = Date.now(),
  durationMs?: number,
): readonly (T | ThinkingStep)[] {
  if (!text) return transcript;
  const last = transcript.at(-1);
  if (isThinkingStep(last) && last.durationMs === undefined && durationMs === undefined)
    return [...transcript.slice(0, -1), { ...last, text: last.text + text }];
  return [...transcript, { kind: "thinking", text, startedAt, durationMs }];
}

export function finishPendingThinking<T extends { kind: string }>(
  transcript: readonly (T | ThinkingStep)[],
  endedAt = Date.now(),
): readonly (T | ThinkingStep)[] {
  const last = transcript.at(-1);
  if (!isThinkingStep(last) || last.durationMs !== undefined) return transcript;
  const durationMs = last.startedAt !== undefined ? Math.max(0, endedAt - last.startedAt) : 0;
  return [...transcript.slice(0, -1), { ...last, durationMs }];
}

interface AnchoredThinking {
  readonly waveToolIds: Set<string>;
  readonly step: ThinkingStep;
  placed: boolean;
}

function collectStoredThinking(events: readonly RunEvent[]): AnchoredThinking[] {
  const anchored: AnchoredThinking[] = [];
  let activeWave: AnchoredThinking[] = [];
  for (const event of events) {
    const thinking = parseThinkingEvent(event);
    if (thinking) {
      const previous = activeWave.at(-1);
      if (previous && previous.waveToolIds.size === 0) previous.step.text += thinking.text;
      else {
        const entry: AnchoredThinking = {
          waveToolIds: new Set(),
          step: { kind: "thinking", text: thinking.text, durationMs: thinking.durationMs },
          placed: false,
        };
        anchored.push(entry);
        activeWave = activeWave.some((item) => item.waveToolIds.size > 0) ? [entry] : [...activeWave, entry];
      }
    } else if (event.kind === "conversation_tool_started") {
      const parsed = startedToolPayload.safeParse(event.payload);
      if (parsed.success) for (const item of activeWave) item.waveToolIds.add(parsed.data.toolCallId);
    }
  }
  return anchored;
}

/**
 * Interleaves stored built-in `agent_thinking` events ahead of the tool wave or answer segment
 * they preceded, anchoring to the entire parallel tool wave so out-of-order tool results never
 * split a group or displace reasoning.
 */
export function interleaveStoredThinking<T extends { kind: string; id?: string }>(
  steps: readonly T[],
  events: readonly RunEvent[],
): (T | ThinkingStep)[] {
  const anchored = collectStoredThinking(events);
  if (!anchored.length) return [...steps];
  const out: (T | ThinkingStep)[] = [];
  let segmentStart = 0;
  for (const step of steps) {
    if (step.kind === "tool" && step.id) {
      const matched = anchored.filter((item) => !item.placed && item.waveToolIds.has(step.id!));
      for (const item of matched) item.placed = true;
      if (matched.length) out.splice(segmentStart, 0, ...matched.map((item) => item.step));
      out.push(step);
      segmentStart = out.length;
    } else {
      out.push(step);
    }
  }
  const remaining = anchored.filter((item) => !item.placed).map((item) => item.step);
  if (remaining.length) out.splice(segmentStart, 0, ...remaining);
  return out;
}
