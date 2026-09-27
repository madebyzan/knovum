export function highlightedSnippet(value: string) {
  const parts: Array<{ text: string; match: boolean }> = [];
  let rest = value;
  while (rest) {
    const start = rest.indexOf("‹");
    if (start < 0) {
      parts.push({ text: rest, match: false });
      break;
    }
    if (start > 0) parts.push({ text: rest.slice(0, start), match: false });
    const end = rest.indexOf("›", start + 1);
    if (end < 0) {
      parts.push({ text: rest.slice(start + 1), match: true });
      break;
    }
    parts.push({ text: rest.slice(start + 1, end), match: true });
    rest = rest.slice(end + 1);
  }
  return parts;
}
