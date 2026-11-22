/**
 * Finds `rawNeedle` inside `haystack` using direct substring matching first, then
 * whitespace-collapsed case-insensitive matching so quotes survive line-wrap differences.
 */
export function findNormalizedTextRange(
  haystack: string,
  rawNeedle: string,
): { start: number; end: number } | null {
  const exact = rawNeedle.trim();
  if (!exact) return null;
  const direct = haystack.indexOf(exact);
  if (direct >= 0) return { start: direct, end: direct + exact.length };
  const target = exact.replace(/\s+/gu, " ").toLowerCase();
  const chars: string[] = [];
  const map: number[] = [];
  let inSpace = false;
  for (let i = 0; i < haystack.length; i += 1) {
    const ch = haystack[i]!;
    if (/\s/u.test(ch)) {
      if (!inSpace && chars.length > 0) {
        chars.push(" ");
        map.push(i);
        inSpace = true;
      }
    } else {
      chars.push(ch.toLowerCase());
      map.push(i);
      inSpace = false;
    }
  }
  const idx = chars.join("").trimEnd().indexOf(target);
  if (idx < 0) return null;
  const start = map[idx]!;
  const end = (map[idx + target.length - 1] ?? start) + 1;
  return { start, end };
}
