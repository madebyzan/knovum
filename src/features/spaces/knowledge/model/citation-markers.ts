const FENCE_OPEN_PATTERN = /^(`{3,}|~{3,})/u;

interface FenceState {
  markerChar: string;
  markerLength: number;
}

/** Restores `[Cn]` escaped by Markdown serializers (`\[C1\]`) outside code fences and spans. */
export function unescapeCitationMarkers(markdown: string): string {
  return rewriteOutsideCode(markdown, (line) =>
    rewriteTextRuns(line, (segment) => segment.replace(/\\\[C([1-9]\d*)\\\]/gu, "[C$1]")),
  );
}

/** Removes internal `[Cn]` citation markers outside fenced code blocks and inline code spans. */
export function stripCitationMarkers(markdown: string): string {
  return rewriteOutsideCode(markdown, (line) =>
    rewriteTextRuns(line, (segment) => segment.replace(/[ \t]*\[C[1-9]\d*\]/gu, "")),
  );
}

function rewriteOutsideCode(markdown: string, transformLine: (line: string) => string): string {
  let fence: FenceState | null = null;
  return markdown
    .split("\n")
    .map((line) => {
      const trimmed = line.trimStart();
      if (fence) {
        if (isClosingFence(trimmed, fence)) fence = null;
        return line;
      }
      const opened = openingFence(trimmed);
      if (opened) {
        fence = opened;
        return line;
      }
      return transformLine(line);
    })
    .join("\n");
}

function rewriteTextRuns(line: string, transformText: (segment: string) => string): string {
  let output = "";
  let cursor = 0;
  let plainStart = 0;
  while (cursor < line.length) {
    if (line[cursor] !== "`") {
      cursor += 1;
      continue;
    }
    const ticks = countTicks(line, cursor);
    const close = findClosingTicks(line, cursor + ticks, ticks);
    if (close === -1) {
      cursor += ticks;
      continue;
    }
    if (cursor > plainStart) output += transformText(line.slice(plainStart, cursor));
    output += line.slice(cursor, close + ticks);
    cursor = close + ticks;
    plainStart = cursor;
  }
  if (plainStart < line.length) output += transformText(line.slice(plainStart));
  return output;
}

function openingFence(trimmed: string): FenceState | null {
  const match = FENCE_OPEN_PATTERN.exec(trimmed);
  if (!match) return null;
  return { markerChar: match[1]![0]!, markerLength: match[1]!.length };
}

function isClosingFence(trimmed: string, fence: FenceState): boolean {
  let count = 0;
  while (count < trimmed.length && trimmed[count] === fence.markerChar) count += 1;
  return count >= fence.markerLength && trimmed.slice(count).trim().length === 0;
}

function countTicks(line: string, start: number): number {
  let count = 0;
  while (start + count < line.length && line[start + count] === "`") count += 1;
  return count;
}

function findClosingTicks(line: string, start: number, ticks: number): number {
  let index = start;
  while (index < line.length) {
    if (line[index] === "`") {
      const run = countTicks(line, index);
      if (run === ticks) return index;
      index += run;
    } else {
      index += 1;
    }
  }
  return -1;
}
