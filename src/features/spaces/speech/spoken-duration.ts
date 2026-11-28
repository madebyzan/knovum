import { formatDurationMs } from "../media/format-duration";

const CJK_RE = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}]/gu;

/**
 * Estimates natural narration playback duration from spoken text.
 * Uses broadcast pacing benchmarks: ~220 CJK characters/min and ~150 Latin words/min.
 */
export function estimateSpokenDurationMs(text: string): number {
  const clean = text.trim();
  if (!clean) return 0;
  const cjkCount = clean.match(CJK_RE)?.length ?? 0;
  const nonCjk = clean.replace(CJK_RE, " ").trim();
  const latinWords = nonCjk ? nonCjk.split(/\s+/u).filter(Boolean).length : 0;
  const minutes = cjkCount / 220 + latinWords / 150;
  return Math.max(1_000, Math.round(minutes * 60_000));
}

export function formatEstimatedDuration(text: string): string {
  const ms = estimateSpokenDurationMs(text);
  return ms > 0 ? `~${formatDurationMs(ms)}` : "";
}
