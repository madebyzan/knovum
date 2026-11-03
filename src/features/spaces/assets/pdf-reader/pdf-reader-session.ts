import { pdfFindAnchor } from "./pdf-find-anchor";

/** Directory the build copies the pdf.js worker, decoders and font data into; see `vite.config.ts`. */
const RESOURCES = "/pdfjs";
const MIN_SCALE = 0.25;
const MAX_SCALE = 5;
const ZOOM_STEP = 1.2;
const FIND_NOT_FOUND = 1;

export interface PdfReaderOpening {
  container: HTMLDivElement;
  viewer: HTMLDivElement;
  url: string;
  page: number;
  passage: string | undefined;
  onPageChange: (page: number, pageCount: number) => void;
}

export interface PdfReaderSession {
  goToPage(page: number): void;
  zoom(direction: 1 | -1): void;
  fitWidth(): void;
  close(): Promise<void>;
}

/** pdf.js is large and only a reader needs it, so it loads with the first document opened. */
async function loadPdfJs() {
  const pdfjs = await import("pdfjs-dist/legacy/build/pdf.mjs");
  // The viewer bundle reads the core library from this global at evaluation time.
  Object.assign(globalThis, { pdfjsLib: pdfjs });
  pdfjs.GlobalWorkerOptions.workerSrc = `${RESOURCES}/pdf.worker.min.mjs`;
  const viewer = await import("pdfjs-dist/legacy/web/pdf_viewer.mjs");
  await import("pdfjs-dist/legacy/web/pdf_viewer.css");
  return { pdfjs, viewer };
}

/**
 * Opens a document at a cited page and marks the cited passage. The view, its text layer and the
 * search are pdf.js's own; this module only says where to look. A page without a text layer, such
 * as an unrecognized scan, has nothing to mark and is simply shown.
 */
export async function openPdfReader(opening: PdfReaderOpening): Promise<PdfReaderSession> {
  const { pdfjs, viewer: components } = await loadPdfJs();
  const eventBus = new components.EventBus();
  const linkService = new components.PDFLinkService({ eventBus });
  const findController = new components.PDFFindController({ eventBus, linkService });
  const viewer = new components.PDFViewer({
    container: opening.container,
    viewer: opening.viewer,
    eventBus,
    linkService,
    findController,
    removePageBorders: true,
  });
  linkService.setViewer(viewer);

  const task = pdfjs.getDocument({
    url: opening.url,
    cMapUrl: `${RESOURCES}/cmaps/`,
    cMapPacked: true,
    standardFontDataUrl: `${RESOURCES}/standard_fonts/`,
    wasmUrl: `${RESOURCES}/wasm/`,
    iccUrl: `${RESOURCES}/iccs/`,
    enableXfa: false,
  });
  const document = await task.promise;

  const find = (query: string) => {
    eventBus.dispatch("find", {
      source: null,
      type: "",
      query,
      caseSensitive: false,
      entireWord: false,
      highlightAll: false,
      findPrevious: false,
      matchDiacritics: false,
    });
  };
  const passage = opening.passage?.replace(/\s+/g, " ").trim();
  let anchor = passage ? pdfFindAnchor(passage) : undefined;
  eventBus.on("updatefindcontrolstate", ({ state }: { state: number }) => {
    if (state !== FIND_NOT_FOUND || !anchor || anchor === passage) return;
    const query = anchor;
    anchor = undefined;
    find(query);
  });
  eventBus.on("pagechanging", ({ pageNumber }: { pageNumber: number }) => {
    opening.onPageChange(pageNumber, document.numPages);
  });
  eventBus.on("pagesinit", () => {
    viewer.currentScaleValue = "page-width";
    const page = Math.min(Math.max(opening.page, 1), document.numPages);
    viewer.currentPageNumber = page;
    opening.onPageChange(page, document.numPages);
    if (passage) find(passage);
  });
  viewer.setDocument(document);
  linkService.setDocument(document);

  return {
    goToPage(page) {
      viewer.currentPageNumber = Math.min(Math.max(page, 1), document.numPages);
    },
    zoom(direction) {
      const scale = viewer.currentScale * ZOOM_STEP ** direction;
      viewer.currentScale = Math.min(Math.max(scale, MIN_SCALE), MAX_SCALE);
    },
    fitWidth() {
      viewer.currentScaleValue = "page-width";
    },
    async close() {
      viewer.setDocument(null);
      linkService.setDocument(null);
      await task.destroy();
    },
  };
}
