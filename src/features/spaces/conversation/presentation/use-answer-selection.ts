import { useTextSelection } from "@vueuse/core";
import { computed, type ShallowRef } from "vue";

/** Half the selection bar's width: keeps the bar inside the message when a selection reaches an edge. */
const BAR_HALF_WIDTH = 72;

/**
 * The text a person selected inside one answer, and where its bar goes. Positions are relative to
 * `container`, which scrolls with the selection, so the bar stays on the text.
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
  const text = computed(() => (range.value ? rangeText(range.value) : ""));
  const anchor = computed(() => {
    const host = container.value;
    if (!range.value || !text.value || !host) return undefined;
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

  return { text, anchor, clear: () => selection.value?.removeAllRanges() };
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

/**
 * The selected words as the reader sees them, one line per block. Citation badges are left out:
 * WebKit includes `user-select: none` text in a selection, and a stray "2" would end up in the note.
 */
export function rangeText(range: Range) {
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
