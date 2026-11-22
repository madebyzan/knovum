export interface PdfNormalizedRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface BoxLike {
  left: number;
  top: number;
  right: number;
  bottom: number;
  width: number;
  height: number;
}

const MIN_SPAN = 0.002;
const LINE_MERGE_TOLERANCE = 0.008;

function clampUnit(value: number): number {
  return Math.min(1, Math.max(0, Number(value.toFixed(5))));
}

/** Converts a client-space box into normalized [0..1] coordinates relative to a PDF page box. */
export function normalizeBoxToPage(box: BoxLike, pageBox: BoxLike): PdfNormalizedRect | undefined {
  if (pageBox.width <= 0 || pageBox.height <= 0 || box.width <= 0 || box.height <= 0) return undefined;
  const x = clampUnit((box.left - pageBox.left) / pageBox.width);
  const y = clampUnit((box.top - pageBox.top) / pageBox.height);
  const right = clampUnit((box.right - pageBox.left) / pageBox.width);
  const bottom = clampUnit((box.bottom - pageBox.top) / pageBox.height);
  const width = Number((right - x).toFixed(5));
  const height = Number((bottom - y).toFixed(5));
  return width >= MIN_SPAN && height >= MIN_SPAN ? { x, y, width, height } : undefined;
}

/** Merges fragmented text-layer boxes on the same line into clean continuous highlight bands. */
export function mergeLineRects(rects: readonly PdfNormalizedRect[]): PdfNormalizedRect[] {
  if (rects.length <= 1) return [...rects];
  const sorted = [...rects].toSorted((a, b) =>
    Math.abs(a.y - b.y) > LINE_MERGE_TOLERANCE ? a.y - b.y : a.x - b.x,
  );
  const merged: PdfNormalizedRect[] = [];
  for (const current of sorted) {
    const prev = merged.at(-1);
    const sameLine =
      prev &&
      Math.abs(prev.y - current.y) <= LINE_MERGE_TOLERANCE &&
      Math.abs(prev.height - current.height) <= LINE_MERGE_TOLERANCE * 2 &&
      current.x <= prev.x + prev.width + 0.015;
    if (sameLine) {
      const right = Math.max(prev.x + prev.width, current.x + current.width);
      const bottom = Math.max(prev.y + prev.height, current.y + current.height);
      prev.x = Math.min(prev.x, current.x);
      prev.y = Math.min(prev.y, current.y);
      prev.width = Number((right - prev.x).toFixed(5));
      prev.height = Number((bottom - prev.y).toFixed(5));
    } else {
      merged.push({ ...current });
    }
  }
  return merged.slice(0, 256);
}

/** Extracts normalized [0..1] highlight rects for a DOM selection range on a pdf.js page element. */
export function extractNormalizedPdfRects(range: Range, pageElement: HTMLElement): PdfNormalizedRect[] {
  const pageBox = pageElement.getBoundingClientRect();
  if (pageBox.width <= 0 || pageBox.height <= 0) return [];
  const raw: PdfNormalizedRect[] = [];
  for (const clientRect of range.getClientRects()) {
    const normalized = normalizeBoxToPage(clientRect, pageBox);
    if (normalized) raw.push(normalized);
  }
  return mergeLineRects(raw);
}
