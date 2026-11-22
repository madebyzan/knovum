import {
  dispatchPdfFind,
  loadPdfJs,
  PDF_HIGHLIGHT_COLORS,
  PDF_RESOURCES,
  resolveColorParam,
  resolveEditorMode,
  resolveSizeParam,
  type PdfReaderOpening,
  type PdfReaderSession,
  type PdfSearchMatches,
} from "./pdf-editor-params";
import { pdfFindAnchor } from "./pdf-find-anchor";
import { loadPdfOutline } from "./pdf-outline";
import { renderPdfPageThumbnail, renderPdfRegionPng } from "./pdf-region-render";

const MIN_SCALE = 0.25;
const MAX_SCALE = 5;
const ZOOM_STEP = 1.15;
const FIND_NOT_FOUND = 1;

interface AnnotationStorageCallbacks {
  onSetModified: (() => void) | null;
  onResetModified: (() => void) | null;
  resetModified(): void;
}

interface PdfViewerEditorHost {
  annotationEditorUIManager?: { undo(): void; redo(): void; delete(): void } | null;
}

/**
 * Opens a PDF with Mozilla pdf.js text layer, find controller, bookmark outline, thumbnail & crop
 * renderers, spread/rotation controls, and native AnnotationEditor + ISO PDF save.
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
    annotationEditorMode: pdfjs.AnnotationEditorType.NONE,
    annotationEditorHighlightColors: PDF_HIGHLIGHT_COLORS,
  });
  linkService.setViewer(viewer);

  const task = pdfjs.getDocument({
    url: opening.url,
    cMapUrl: `${PDF_RESOURCES}/cmaps/`,
    cMapPacked: true,
    standardFontDataUrl: `${PDF_RESOURCES}/standard_fonts/`,
    wasmUrl: `${PDF_RESOURCES}/wasm/`,
    iccUrl: `${PDF_RESOURCES}/iccs/`,
    enableXfa: false,
  });
  const doc = await task.promise;

  const storage = doc.annotationStorage as unknown as AnnotationStorageCallbacks;
  storage.onSetModified = () => opening.onModifiedChange?.(true);
  storage.onResetModified = () => opening.onModifiedChange?.(false);

  const passage = opening.passage?.replace(/\s+/g, " ").trim();
  let anchor = passage ? pdfFindAnchor(passage) : undefined;
  eventBus.on(
    "updatefindcontrolstate",
    ({ state, matchesCount }: { state: number; matchesCount?: PdfSearchMatches }) => {
      if (matchesCount) opening.onMatches?.(matchesCount);
      if (state !== FIND_NOT_FOUND || !anchor || anchor === passage) return;
      const query = anchor;
      anchor = undefined;
      dispatchPdfFind(eventBus, query);
    },
  );
  eventBus.on("updatefindmatchescount", ({ matchesCount }: { matchesCount: PdfSearchMatches }) => {
    opening.onMatches?.(matchesCount);
  });
  eventBus.on("pagechanging", ({ pageNumber }: { pageNumber: number }) =>
    opening.onPageChange(pageNumber, doc.numPages),
  );
  eventBus.on("pagerendered", ({ pageNumber }: { pageNumber: number }) =>
    opening.onPageRendered?.(pageNumber),
  );
  eventBus.on("scalechanging", ({ scale }: { scale: number }) =>
    opening.onScaleChange?.(Math.round(scale * 100)),
  );
  eventBus.on("pagesinit", () => {
    viewer.currentScaleValue = "page-width";
    const page = Math.min(Math.max(opening.page, 1), doc.numPages);
    viewer.currentPageNumber = page;
    opening.onPageChange(page, doc.numPages);
    opening.onScaleChange?.(Math.round(viewer.currentScale * 100));
    if (passage) dispatchPdfFind(eventBus, passage);
  });

  viewer.setDocument(doc);
  linkService.setDocument(doc);
  void loadPdfOutline(doc).then((entries) => opening.onOutline?.(entries));

  return {
    goToPage: (page) => {
      viewer.currentPageNumber = Math.min(Math.max(page, 1), doc.numPages);
    },
    navigate(page, nextPassage) {
      viewer.currentPageNumber = Math.min(Math.max(page, 1), doc.numPages);
      const normalized = nextPassage?.replace(/\s+/g, " ").trim();
      anchor = normalized ? pdfFindAnchor(normalized) : undefined;
      if (normalized) dispatchPdfFind(eventBus, normalized);
    },
    zoom(direction) {
      const scale = viewer.currentScale * ZOOM_STEP ** direction;
      viewer.currentScale = Math.min(Math.max(scale, MIN_SCALE), MAX_SCALE);
    },
    fitWidth: () => {
      viewer.currentScaleValue = "page-width";
    },
    fitPage: () => {
      viewer.currentScaleValue = "page-fit";
    },
    setSpreadMode: (mode) => {
      viewer.spreadMode = mode;
    },
    rotate: (delta) => {
      viewer.pagesRotation = (viewer.pagesRotation + delta + 360) % 360;
    },
    find(query, direction) {
      anchor = undefined;
      if (!query.trim()) {
        dispatchPdfFind(eventBus, "");
        opening.onMatches?.({ current: 0, total: 0 });
        return;
      }
      dispatchPdfFind(eventBus, query, {
        type: direction ? "again" : "",
        findPrevious: direction === "prev",
        highlightAll: true,
      });
    },
    clearFind() {
      anchor = undefined;
      dispatchPdfFind(eventBus, "");
      opening.onMatches?.({ current: 0, total: 0 });
    },
    setAnnotationTool: (tool) => {
      viewer.annotationEditorMode = { mode: resolveEditorMode(pdfjs.AnnotationEditorType, tool) };
    },
    setEditorColor: (tool, hex) => {
      eventBus.dispatch("switchannotationeditorparams", {
        type: resolveColorParam(pdfjs.AnnotationEditorParamsType, tool),
        value: hex,
      });
    },
    setEditorSize: (tool, value) => {
      eventBus.dispatch("switchannotationeditorparams", {
        type: resolveSizeParam(pdfjs.AnnotationEditorParamsType, tool),
        value,
      });
    },
    undo: () => (viewer as unknown as PdfViewerEditorHost).annotationEditorUIManager?.undo(),
    redo: () => (viewer as unknown as PdfViewerEditorHost).annotationEditorUIManager?.redo(),
    deleteSelected: () => (viewer as unknown as PdfViewerEditorHost).annotationEditorUIManager?.delete(),
    renderThumbnail: (page, canvas, width) => renderPdfPageThumbnail(doc, page, canvas, width),
    renderRegionPng: (page, rect) => renderPdfRegionPng(doc, page, rect, viewer.pagesRotation),
    async saveDocument() {
      const bytes = await doc.saveDocument();
      storage.resetModified();
      return new Uint8Array(bytes);
    },
    async close() {
      storage.onSetModified = null;
      storage.onResetModified = null;
      viewer.setDocument(null);
      linkService.setDocument(null);
      await task.destroy();
    },
  };
}
