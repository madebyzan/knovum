export const MAX_KNOWLEDGE_PASSAGE_BYTES = 48 * 1024;

export interface KnowledgePassage {
  text: string;
  startOffset: number;
  endOffset: number;
}

/**
 * Mirrors the Rust durable splitter: 48 KiB UTF-8 windows, preferring whitespace
 * in the final 40% of a full window. Offsets are JavaScript UTF-16 units.
 */
export function splitKnowledgePassages(value: string): KnowledgePassage[] {
  const passages: KnowledgePassage[] = [];
  let cursor = 0;
  while (cursor < value.length) {
    cursor = skipWhitespace(value, cursor);
    if (cursor >= value.length) break;
    const window = fitPassageWindow(value, cursor);
    const breakAt = passageBreak(value, cursor, window.end, window.bytes);
    appendPassage(passages, value, cursor, breakAt);
    cursor = breakAt;
  }
  return passages;
}

function skipWhitespace(value: string, start: number) {
  let cursor = start;
  while (cursor < value.length && isWhitespaceAt(value, cursor)) cursor += scalarWidthUtf16(value, cursor);
  return cursor;
}

function fitPassageWindow(value: string, start: number) {
  let end = start;
  let bytes = 0;
  while (end < value.length) {
    const codePoint = value.codePointAt(end);
    if (codePoint === undefined) break;
    const byteWidth = utf8Width(codePoint);
    if (bytes + byteWidth > MAX_KNOWLEDGE_PASSAGE_BYTES) break;
    bytes += byteWidth;
    end += codePoint > 0xffff ? 2 : 1;
  }
  if (end === start) throw new Error("Knowledge passage byte budget is too small for one Unicode scalar");
  return { end, bytes };
}

function passageBreak(value: string, start: number, end: number, bytes: number) {
  if (end >= value.length) return end;
  const preferred = lastWhitespaceBreak(value, start, end, Math.floor((bytes * 3) / 5));
  return preferred > start ? preferred : end;
}

function appendPassage(passages: KnowledgePassage[], value: string, start: number, end: number) {
  const text = value.slice(start, end).trimEnd();
  if (text) passages.push({ text, startOffset: start, endOffset: start + text.length });
}

function lastWhitespaceBreak(value: string, start: number, end: number, preferredByte: number) {
  let cursor = start;
  let bytes = 0;
  let lastBreak = -1;
  while (cursor < end) {
    const codePoint = value.codePointAt(cursor);
    if (codePoint === undefined) break;
    if (bytes >= preferredByte && isWhitespaceCodePoint(codePoint)) lastBreak = cursor;
    bytes += utf8Width(codePoint);
    cursor += codePoint > 0xffff ? 2 : 1;
  }
  return lastBreak;
}

function scalarWidthUtf16(value: string, offset: number) {
  return (value.codePointAt(offset) ?? 0) > 0xffff ? 2 : 1;
}

function isWhitespaceAt(value: string, offset: number) {
  const width = scalarWidthUtf16(value, offset);
  return /^\s$/u.test(value.slice(offset, offset + width));
}

function isWhitespaceCodePoint(codePoint: number) {
  return /^\s$/u.test(String.fromCodePoint(codePoint));
}

function utf8Width(codePoint: number) {
  if (codePoint <= 0x7f) return 1;
  if (codePoint <= 0x7ff) return 2;
  if (codePoint <= 0xffff) return 3;
  return 4;
}
