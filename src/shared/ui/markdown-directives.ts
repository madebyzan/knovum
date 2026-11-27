export interface ParsedDirectiveOpen {
  readonly name: string;
  readonly fenceLength: number;
  readonly attrs: Readonly<Record<string, string>>;
}

const DIRECTIVE_OPEN_RE = /^(:{3,})([a-z][a-z0-9-]*)(?:\{([^}]*)\})?[ \t]*$/u;
const ATTR_PAIR_RE = /([a-zA-Z_][a-zA-Z0-9_-]*)(?:=(?:"([^"]*)"|'([^']*)'|([^\s}]+)))?/gu;

function parseDirectiveAttrs(raw: string): Record<string, string> {
  const attrs: Record<string, string> = {};
  for (const match of raw.matchAll(ATTR_PAIR_RE)) {
    const key = match[1];
    if (!key) continue;
    attrs[key] = match[2] ?? match[3] ?? match[4] ?? "true";
  }
  return attrs;
}

export function formatDirectiveAttrs(attrs: Readonly<Record<string, string | number | undefined>>): string {
  const parts: string[] = [];
  for (const [key, value] of Object.entries(attrs)) {
    if (value === undefined || value === "") continue;
    const escaped = String(value).replaceAll('"', '\\"');
    parts.push(`${key}="${escaped}"`);
  }
  return parts.length ? `{${parts.join(" ")}}` : "";
}

export function parseDirectiveOpenLine(line: string): ParsedDirectiveOpen | null {
  const match = DIRECTIVE_OPEN_RE.exec(line.trim());
  if (!match) return null;
  const fence = match[1]!;
  const name = match[2]!.toLowerCase();
  const attrs = match[3] ? parseDirectiveAttrs(match[3]) : {};
  return { name, fenceLength: fence.length, attrs };
}

export function isDirectiveCloseLine(line: string, minFenceLength: number): boolean {
  const trimmed = line.trim();
  return trimmed.length >= minFenceLength && /^:+$/u.test(trimmed);
}

export interface DirectiveBlockMatch {
  readonly raw: string;
  readonly name: string;
  readonly attrs: Readonly<Record<string, string>>;
  readonly body: string;
}

function findDirectiveCloseLine(lines: readonly string[], fenceLength: number): number {
  let depth = 1;
  for (let i = 1; i < lines.length; i += 1) {
    const line = lines[i]!;
    if (parseDirectiveOpenLine(line)) {
      depth += 1;
    } else if (isDirectiveCloseLine(line, fenceLength)) {
      depth -= 1;
      if (depth === 0) return i;
    }
  }
  return -1;
}

/** Matches a CommonMark container directive (`:::name{...}\n...\n:::`) at index 0 of `src`. */
export function matchDirectiveBlock(src: string, expectedName?: string): DirectiveBlockMatch | null {
  const lines = src.split(/\r?\n/u);
  const open = lines[0] === undefined ? null : parseDirectiveOpenLine(lines[0]);
  if (!open || (expectedName && open.name !== expectedName)) return null;

  const endLineIndex = findDirectiveCloseLine(lines, open.fenceLength);
  if (endLineIndex === -1) return null;

  const rawBase = lines.slice(0, endLineIndex + 1).join("\n");
  const trailingNewline = src.slice(rawBase.length).match(/^\r?\n/u)?.[0] ?? "";
  return {
    raw: rawBase + trailingNewline,
    name: open.name,
    attrs: open.attrs,
    body: lines.slice(1, endLineIndex).join("\n"),
  };
}
