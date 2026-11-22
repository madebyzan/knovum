import type { PdfNormalizedRect } from "../../reading/pdf-rect-geometry";
import type { PdfBookmark } from "./pdf-outline";
import type { RenderedPdfCrop } from "./pdf-region-render";

export type PdfAnnotationTool = "none" | "highlight" | "freetext" | "ink";
export type PdfSpreadMode = 0 | 1 | 2;

export interface PdfSearchMatches {
  current: number;
  total: number;
}

export interface PdfReaderOpening {
  container: HTMLDivElement;
  viewer: HTMLDivElement;
  url: string;
  page: number;
  passage: string | undefined;
  onPageChange: (page: number, pageCount: number) => void;
  onPageRendered?: (page: number) => void;
  onScaleChange?: (percent: number) => void;
  onOutline?: (bookmarks: readonly PdfBookmark[]) => void;
  onMatches?: (matches: PdfSearchMatches) => void;
  onModifiedChange?: (modified: boolean) => void;
}

export interface PdfReaderSession {
  goToPage(page: number): void;
  navigate(page: number, passage?: string | undefined): void;
  zoom(direction: 1 | -1): void;
  fitWidth(): void;
  fitPage(): void;
  setSpreadMode(mode: PdfSpreadMode): void;
  rotate(deltaDegrees: 90 | -90): void;
  find(query: string, direction?: "next" | "prev"): void;
  clearFind(): void;
  setAnnotationTool(tool: PdfAnnotationTool): void;
  setEditorColor(tool: Exclude<PdfAnnotationTool, "none">, hex: string): void;
  setEditorSize(tool: "freetext" | "ink", value: number): void;
  undo(): void;
  redo(): void;
  deleteSelected(): void;
  renderThumbnail(pageNumber: number, canvas: HTMLCanvasElement, targetWidth: number): Promise<void>;
  renderRegionPng(pageNumber: number, rect: PdfNormalizedRect): Promise<RenderedPdfCrop>;
  saveDocument(): Promise<Uint8Array>;
  close(): Promise<void>;
}

export const PDF_RESOURCES = "/pdfjs";
export const PDF_HIGHLIGHT_COLORS = "yellow=#FEF08A,green=#BBF7D0,blue=#BFDBFE,pink=#FBCFE8";
export const PDF_HIGHLIGHT_SWATCHES = ["#FEF08A", "#BBF7D0", "#BFDBFE", "#FBCFE8"] as const;
export const PDF_INK_SWATCHES = ["#1F2328", "#DC2626", "#2563EB", "#15803D"] as const;
export const PDF_TEXT_SIZES = [12, 14, 18, 24] as const;
export const PDF_INK_WIDTHS = [2, 4, 7] as const;

export async function loadPdfJs() {
  const pdfjs = await import("pdfjs-dist/legacy/build/pdf.mjs");
  Object.assign(globalThis, { pdfjsLib: pdfjs });
  pdfjs.GlobalWorkerOptions.workerSrc = `${PDF_RESOURCES}/pdf.worker.min.mjs`;
  const viewer = await import("pdfjs-dist/legacy/web/pdf_viewer.mjs");
  await import("pdfjs-dist/legacy/web/pdf_viewer.css");
  return { pdfjs, viewer };
}

export function dispatchPdfFind(
  eventBus: { dispatch(name: string, data: object): void },
  query: string,
  options: { type?: "" | "again"; findPrevious?: boolean; highlightAll?: boolean } = {},
) {
  eventBus.dispatch("find", {
    source: null,
    type: options.type ?? "",
    query,
    caseSensitive: false,
    entireWord: false,
    highlightAll: options.highlightAll ?? false,
    findPrevious: options.findPrevious ?? false,
    matchDiacritics: false,
  });
}

interface EditorTypeTable {
  NONE: number;
  HIGHLIGHT: number;
  FREETEXT: number;
  INK: number;
}

interface EditorParamTable {
  HIGHLIGHT_COLOR: number;
  FREETEXT_COLOR: number;
  FREETEXT_SIZE: number;
  INK_COLOR: number;
  INK_THICKNESS: number;
}

export function resolveEditorMode(types: EditorTypeTable, tool: PdfAnnotationTool): number {
  if (tool === "highlight") return types.HIGHLIGHT;
  if (tool === "freetext") return types.FREETEXT;
  if (tool === "ink") return types.INK;
  return types.NONE;
}

export function resolveColorParam(
  params: EditorParamTable,
  tool: Exclude<PdfAnnotationTool, "none">,
): number {
  if (tool === "highlight") return params.HIGHLIGHT_COLOR;
  return tool === "freetext" ? params.FREETEXT_COLOR : params.INK_COLOR;
}

export function resolveSizeParam(params: EditorParamTable, tool: "freetext" | "ink"): number {
  return tool === "freetext" ? params.FREETEXT_SIZE : params.INK_THICKNESS;
}
