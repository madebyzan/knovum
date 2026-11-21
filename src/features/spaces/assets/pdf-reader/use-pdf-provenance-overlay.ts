import { computed, watch, type ComputedRef, type Ref } from "vue";

import type { PdfProvenanceLink } from "./use-pdf-source-links";

const LAYER_CLASS = "knovum-pdf-provenance-layer";

function buildMarkElement(
  link: PdfProvenanceLink,
  rect: { x: number; y: number; width: number; height: number },
  activeQuote: string | undefined,
  onSelect: (targetNodeId: string) => void,
): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = link.targetKind === "image" ? "knovum-pdf-crop-mark" : "knovum-pdf-highlight-mark";
  if (activeQuote && link.quote.includes(activeQuote)) {
    button.dataset.active = "true";
  }
  button.dataset.readerIgnoreSelection = "";
  button.title = link.targetTitle;
  button.style.left = `${rect.x * 100}%`;
  button.style.top = `${rect.y * 100}%`;
  button.style.width = `${rect.width * 100}%`;
  button.style.height = `${rect.height * 100}%`;
  button.addEventListener("click", (event) => {
    event.stopPropagation();
    onSelect(link.targetNodeId);
  });
  return button;
}

/**
 * Synchronizes persistent bidirectional provenance highlights onto pdf.js `.page` elements.
 * Clicking any painted highlight reveals and selects the linked Knowledge Node on the Canvas.
 */
export function usePdfProvenanceOverlay(
  viewerRef: Readonly<Ref<HTMLDivElement | null>>,
  links: ComputedRef<readonly PdfProvenanceLink[]>,
  activeQuote: ComputedRef<string | undefined>,
  onSelectTarget: (targetNodeId: string) => void,
) {
  const linksByPage = computed(() => {
    const grouped = new Map<number, PdfProvenanceLink[]>();
    for (const link of links.value) {
      if (!link.rects.length) continue;
      const bucket = grouped.get(link.page);
      if (bucket) bucket.push(link);
      else grouped.set(link.page, [link]);
    }
    return grouped;
  });

  function syncPage(pageNumber: number) {
    const viewer = viewerRef.value;
    if (!viewer) return;
    const pageEl = viewer.querySelector<HTMLElement>(`.page[data-page-number="${pageNumber}"]`);
    if (!pageEl) return;
    pageEl.querySelector(`.${LAYER_CLASS}`)?.remove();

    const pageLinks = linksByPage.value.get(pageNumber);
    if (!pageLinks?.length) return;

    const layer = document.createElement("div");
    layer.className = LAYER_CLASS;
    layer.dataset.readerIgnoreSelection = "";
    const quote = activeQuote.value?.trim();
    for (const link of pageLinks) {
      for (const rect of link.rects) {
        layer.append(buildMarkElement(link, rect, quote, onSelectTarget));
      }
    }
    pageEl.append(layer);
  }

  function syncAllRenderedPages() {
    const viewer = viewerRef.value;
    if (!viewer) return;
    for (const pageEl of viewer.querySelectorAll<HTMLElement>(".page[data-page-number]")) {
      const page = Number(pageEl.dataset.pageNumber);
      if (Number.isInteger(page) && page > 0) syncPage(page);
    }
  }

  watch([links, activeQuote], () => syncAllRenderedPages(), { flush: "post" });

  return { syncPage, syncAllRenderedPages };
}
