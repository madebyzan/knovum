export interface ActiveMentionToken {
  readonly start: number;
  readonly end: number;
  readonly query: string;
}

const MENTION_PATTERN = /(?:^|\s)@([^\s@]*)$/u;

export function findActiveMentionToken(text: string, caret: number): ActiveMentionToken | null {
  const clamped = Math.max(0, Math.min(caret, text.length));
  const prefix = text.slice(0, clamped);
  const match = MENTION_PATTERN.exec(prefix);
  if (!match) return null;
  const query = match[1] ?? "";
  const atIndex = prefix.lastIndexOf("@");
  if (atIndex < 0) return null;
  return { start: atIndex, end: clamped, query };
}

export function stripMentionToken(text: string, token: ActiveMentionToken): { text: string; caret: number } {
  const before = text.slice(0, token.start);
  const after = text.slice(token.end);
  const needsSpace = before.length > 0 && !/\s$/u.test(before) && after.length > 0 && !/^\s/u.test(after);
  const next = `${before}${needsSpace ? " " : ""}${after.replace(/^\s+/u, before.endsWith(" ") ? "" : " ").trimStart()}`;
  const caret = before.length + (needsSpace ? 1 : 0);
  return { text: next, caret: Math.min(caret, next.length) };
}
