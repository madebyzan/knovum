const ANCHOR_CHARACTERS = 48;

/**
 * The opening of a passage, cut on a word boundary. A parser and a viewer break lines and hyphenate
 * differently, so a long passage may not match the page verbatim while its opening still does.
 */
export function pdfFindAnchor(passage: string): string {
  const text = passage.replace(/\s+/g, " ").trim();
  if (text.length <= ANCHOR_CHARACTERS) return text;
  const cut = text.lastIndexOf(" ", ANCHOR_CHARACTERS);
  return text.slice(0, cut > 0 ? cut : ANCHOR_CHARACTERS);
}
