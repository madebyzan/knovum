import { convertFileSrc } from "@tauri-apps/api/core";
import { onBeforeUnmount, shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { getCanvasEditorSession } from "../../canvas/editor-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { PdfNormalizedRect } from "../../reading/pdf-rect-geometry";
import { useSpaceStore } from "../../session/space-store";
import { saveEditableSpaceAsset } from "../asset-operations";
import { capturePdfAreaToCanvas } from "./pdf-area-capture";
import type { PdfReaderSession, PdfSearchMatches, PdfSpreadMode } from "./pdf-editor-params";
import type { PdfBookmark } from "./pdf-outline";
import { openPdfReader } from "./pdf-reader-session";

export interface PdfLifecycleOptions {
  container: Readonly<Ref<HTMLDivElement | null>>;
  viewer: Readonly<Ref<HTMLDivElement | null>>;
  getNode: () => SpaceCanvasNode | undefined;
  getPath: () => string;
  getInitialPage: () => number | undefined;
  getPassage: () => string | undefined;
  onPageChange: (page: number) => void;
  onPageRendered: (page: number) => void;
}

/** Owns the pdf.js session lifecycle, annotation saving, area cropping, and view/outline/search state. */
export function usePdfReaderLifecycle(options: PdfLifecycleOptions) {
  const { t } = useI18n();
  const store = useSpaceStore();
  const session = shallowRef<PdfReaderSession>();
  const currentPage = shallowRef(options.getInitialPage() ?? 1);
  const pageCount = shallowRef(0);
  const scalePercent = shallowRef(100);
  const spreadMode = shallowRef<PdfSpreadMode>(0);
  const failure = shallowRef<string>();
  const outlineEntries = shallowRef<readonly PdfBookmark[]>([]);
  const findMatches = shallowRef<PdfSearchMatches>({ current: 0, total: 0 });
  const modified = shallowRef(false);
  const saving = shallowRef(false);
  let activeToken = 0;

  async function closeSession() {
    const active = session.value;
    session.value = undefined;
    modified.value = false;
    if (active) await active.close();
  }

  async function start(containerEl: HTMLDivElement, viewerEl: HTMLDivElement, path: string) {
    const token = ++activeToken;
    await closeSession();
    failure.value = undefined;
    outlineEntries.value = [];
    try {
      const opened = await openPdfReader({
        container: containerEl,
        viewer: viewerEl,
        url: convertFileSrc(path),
        page: currentPage.value || (options.getInitialPage() ?? 1),
        passage: options.getPassage(),
        onPageChange(current, total) {
          currentPage.value = current;
          pageCount.value = total;
          options.onPageChange(current);
        },
        onPageRendered: options.onPageRendered,
        onScaleChange: (percent) => {
          scalePercent.value = percent;
        },
        onOutline: (entries) => {
          if (token === activeToken) outlineEntries.value = entries;
        },
        onMatches: (matches) => {
          if (token === activeToken) findMatches.value = matches;
        },
        onModifiedChange: (dirty) => {
          if (token === activeToken) modified.value = dirty;
        },
      });
      if (token === activeToken) {
        session.value = opened;
        spreadMode.value = 0;
      } else {
        await opened.close();
      }
    } catch (error) {
      if (token === activeToken) failure.value = getErrorMessage(error);
    }
  }

  async function savePdf() {
    const active = session.value;
    const spaceId = store.activeSpaceId;
    const node = options.getNode();
    if (!active || !spaceId || !node || saving.value) return;
    saving.value = true;
    try {
      const bytes = await active.saveDocument();
      const refreshed = await saveEditableSpaceAsset(spaceId, node.id, bytes);
      if (refreshed) store.applyNodeContentForSpace(spaceId, node.id, refreshed);
      modified.value = false;
      toast.success(t("common.saved"));
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      saving.value = false;
    }
  }

  async function captureArea(pageNumber: number, rect: PdfNormalizedRect) {
    const node = options.getNode();
    if (!session.value || !node) return;
    try {
      const created = await capturePdfAreaToCanvas(session.value, node, pageNumber, rect);
      if (!created) return;
      toast.success(t("spaces.conversation.capture.saved"), {
        action: {
          label: t("spaces.conversation.capture.show"),
          onClick: () => {
            store.requestReveal([created.id], "focus");
            getCanvasEditorSession().selectNodes([created.id]);
          },
        },
      });
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }

  async function reload() {
    const containerEl = options.container.value;
    const viewerEl = options.viewer.value;
    const path = options.getPath();
    if (containerEl && viewerEl && path) await start(containerEl, viewerEl, path);
  }

  function setSpread(mode: PdfSpreadMode) {
    spreadMode.value = mode;
    session.value?.setSpreadMode(mode);
  }

  watch(
    [options.container, options.viewer, options.getPath],
    ([containerEl, viewerEl, path]) => {
      if (containerEl && viewerEl && path) {
        currentPage.value = options.getInitialPage() ?? 1;
        void start(containerEl, viewerEl, path);
      } else if (session.value) {
        void closeSession();
      }
    },
    { immediate: true, flush: "post" },
  );

  watch(
    () => [options.getInitialPage(), options.getPassage()] as const,
    ([page, passage]) => {
      if (page !== undefined && session.value) session.value.navigate(page, passage);
    },
  );

  onBeforeUnmount(() => {
    activeToken += 1;
    void closeSession();
  });

  return {
    session,
    currentPage,
    pageCount,
    scalePercent,
    spreadMode,
    failure,
    outlineEntries,
    findMatches,
    modified,
    saving,
    savePdf,
    captureArea,
    reload,
    setSpread,
  };
}
