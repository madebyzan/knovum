import { clamp } from "es-toolkit";

/** Truncate by JavaScript UTF-16 code units without splitting a surrogate pair. */
export function truncateUtf16(value: string, maxUnits: number) {
  if (!Number.isSafeInteger(maxUnits) || maxUnits < 0)
    throw new Error("UTF-16 truncation limit must be a safe non-negative integer");
  if (value.length <= maxUnits) return value;
  let units = 0;
  let truncated = "";
  for (const character of value) {
    if (units + character.length > maxUnits) break;
    truncated += character;
    units += character.length;
  }
  return truncated;
}

/** Slice by JavaScript UTF-16 code-unit offsets while keeping both boundaries on scalar boundaries. */
export function sliceUtf16Safe(value: string, start = 0, end = value.length) {
  const normalizedStart = clampIndex(start, value.length);
  const normalizedEnd = clampIndex(end, value.length);
  const safeStart = isLowSurrogate(value.charCodeAt(normalizedStart)) ? normalizedStart + 1 : normalizedStart;
  const safeEnd =
    normalizedEnd > safeStart &&
    isHighSurrogate(value.charCodeAt(normalizedEnd - 1)) &&
    isLowSurrogate(value.charCodeAt(normalizedEnd))
      ? normalizedEnd - 1
      : normalizedEnd;
  return value.slice(safeStart, Math.max(safeStart, safeEnd));
}

function clampIndex(value: number, length: number) {
  if (!Number.isSafeInteger(value)) throw new Error("UTF-16 slice index must be a safe integer");
  return clamp(value, 0, length);
}

function isHighSurrogate(value: number) {
  return value >= 0xd800 && value <= 0xdbff;
}

function isLowSurrogate(value: number) {
  return value >= 0xdc00 && value <= 0xdfff;
}

const utf8Encoder = new TextEncoder();

/** Matches Rust String ordering and SQLite BINARY collation for valid UTF-8 text. */
export function compareUtf8Binary(left: string, right: string): number {
  const leftBytes = utf8Encoder.encode(left);
  const rightBytes = utf8Encoder.encode(right);
  const length = Math.min(leftBytes.length, rightBytes.length);
  for (let index = 0; index < length; index += 1) {
    const difference = (leftBytes[index] ?? 0) - (rightBytes[index] ?? 0);
    if (difference !== 0) return difference < 0 ? -1 : 1;
  }
  return leftBytes.length < rightBytes.length ? -1 : leftBytes.length > rightBytes.length ? 1 : 0;
}
