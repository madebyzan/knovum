import type { PDFDocumentProxy } from "pdfjs-dist/types/src/display/api";

export interface PdfBookmark {
  title: string;
  page: number;
  level: number;
}

type RawOutlineNode = Awaited<ReturnType<PDFDocumentProxy["getOutline"]>>[number];

interface PendingOutlineNode {
  title: string;
  dest: RawOutlineNode["dest"];
  level: number;
}

function collectNodes(nodes: readonly RawOutlineNode[], level: number, out: PendingOutlineNode[]) {
  for (const node of nodes) {
    if (out.length >= 200) return;
    const title = node.title.trim();
    if (title) out.push({ title, dest: node.dest, level });
    if (node.items.length && level < 3) collectNodes(node.items, level + 1, out);
  }
}

async function resolveDestPage(
  doc: PDFDocumentProxy,
  dest: RawOutlineNode["dest"],
): Promise<number | undefined> {
  try {
    const resolved = typeof dest === "string" ? await doc.getDestination(dest) : dest;
    const ref = Array.isArray(resolved) ? resolved[0] : undefined;
    if (!ref || typeof ref !== "object") return undefined;
    const index = await doc.getPageIndex(ref as Parameters<PDFDocumentProxy["getPageIndex"]>[0]);
    return index + 1;
  } catch {
    return undefined;
  }
}

/** Extracts a bounded, flat TOC from a PDF's embedded bookmark tree. */
export async function loadPdfOutline(doc: PDFDocumentProxy): Promise<readonly PdfBookmark[]> {
  try {
    const outline = await doc.getOutline();
    if (!outline?.length) return [];
    const pending: PendingOutlineNode[] = [];
    collectNodes(outline, 0, pending);
    const resolved = await Promise.all(
      pending.map(async (item) => {
        const page = await resolveDestPage(doc, item.dest);
        return page === undefined ? undefined : { title: item.title, page, level: item.level };
      }),
    );
    return resolved.filter((bookmark): bookmark is PdfBookmark => bookmark !== undefined);
  } catch {
    return [];
  }
}
