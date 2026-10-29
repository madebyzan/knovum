import { useTextSelection } from "@vueuse/core";
import { computed, type ShallowRef } from "vue";

import type { AnswerPassage } from "../answer-capture";

/** Half the selection bar's width: keeps the bar inside the message when a selection reaches an edge. */
const BAR_HALF_WIDTH = 72;

/**
 * The passage a person selected inside one answer, and where its bar goes. Positions are relative
 * to `container`, which scrolls with the selection, so the bar stays on the text.
 */
export function useAnswerSelection(container: Readonly<ShallowRef<HTMLElement | null>>) {
  const { selection, ranges } = useTextSelection();
  const range = computed(() => {
    const [candidate] = ranges.value;
    const host = container.value;
    return candidate && !candidate.collapsed && host?.contains(candidate.commonAncestorContainer)
      ? candidate
      : undefined;
  });
  const passage = computed(() =>
    range.value && container.value ? rangePassage(range.value, container.value) : undefined,
  );
  const anchor = computed(() => {
    const host = container.value;
    if (!range.value || !passage.value || !host) return undefined;
    const box = range.value.getBoundingClientRect();
    const frame = host.getBoundingClientRect();
    const center = box.left - frame.left + box.width / 2;
    return {
      left: Math.min(
        Math.max(center, BAR_HALF_WIDTH),
        Math.max(frame.width - BAR_HALF_WIDTH, BAR_HALF_WIDTH),
      ),
      top: box.top - frame.top,
    };
  });

  /** Whether a viewport point is on the selected words, where a press starts dragging them. */
  function covers(point: { x: number; y: number }) {
    return [...(range.value?.getClientRects() ?? [])].some(
      (box) => point.x >= box.left && point.x <= box.right && point.y >= box.top && point.y <= box.bottom,
    );
  }

  return { passage, anchor, covers, clear: () => selection.value?.removeAllRanges() };
}

// Elements that start a new line in the answer's rendered Markdown.
const BLOCKS = new Set([
  "P",
  "LI",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "BLOCKQUOTE",
  "PRE",
  "TR",
  "DIV",
  "TABLE",
  "UL",
  "OL",
]);

/** The selection inside `host` as a passage, or nothing when it holds no words. */
export function rangePassage(range: Range, host: HTMLElement): AnswerPassage | undefined {
  const text = rangeText(range);
  return text ? { text, citations: citedAround(range, host) } : undefined;
}

const BLOCK_SELECTOR = "p, li, h1, h2, h3, h4, h5, h6, blockquote, pre, tr";

/**
 * The citations of every block the selection touches. A citation closes the sentence it backs, and
 * people select the claim without reaching for the badge after it.
 */
function citedAround(range: Range, host: HTMLElement) {
  const around = range.cloneRange();
  const blockOf = (node: Node) => {
    const block = (node instanceof Element ? node : node.parentElement)?.closest(BLOCK_SELECTOR);
    return block && host.contains(block) ? block : undefined;
  };
  const first = blockOf(range.startContainer);
  const last = blockOf(range.endContainer);
  if (first) around.setStartBefore(first);
  if (last) around.setEndAfter(last);
  const badges = around.cloneContents().querySelectorAll<HTMLElement>(".ui-citation[data-citation-index]");
  return [...new Set([...badges].map((badge) => Number(badge.dataset.citationIndex)))];
}

/**
 * The selected words as the reader sees them, one line per block. Citation badges are left out:
 * WebKit includes `user-select: none` text in a selection, and a stray "2" would end up in the note.
 */
function rangeText(range: Range) {
  const fragment = range.cloneContents();
  for (const badge of fragment.querySelectorAll(".ui-citation")) {
    // The space before a badge goes with it, as in `withoutCitationMarkers`: "Q3 [C1]." reads "Q3.".
    const before = badge.previousSibling;
    if (before?.nodeType === Node.TEXT_NODE) before.textContent = before.textContent!.replace(/[ \t]+$/u, "");
    badge.remove();
  }
  const lines: string[] = [""];
  const walk = (node: Node) => {
    if (node.nodeType === Node.TEXT_NODE) lines[lines.length - 1] += node.textContent ?? "";
    else if (node instanceof HTMLBRElement) lines.push("");
    else if (node instanceof HTMLTableCellElement && node.previousElementSibling)
      lines[lines.length - 1] += "\t";
    const block = node instanceof Element && BLOCKS.has(node.tagName);
    if (block) lines.push("");
    node.childNodes.forEach(walk);
    if (block) lines.push("");
  };
  fragment.childNodes.forEach(walk);
  return lines
    .map((line) => line.replace(/[ \t]+$/u, "").replace(/ {2,}/gu, " "))
    .join("\n")
    .replace(/\n{3,}/gu, "\n\n")
    .trim();
}
