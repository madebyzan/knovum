import { shallowRef, type Ref, type ShallowRef } from "vue";

import { normalizeBoxToPage, type PdfNormalizedRect } from "../../reading/pdf-rect-geometry";

export interface AreaMarqueeBox {
  left: number;
  top: number;
  width: number;
  height: number;
}

interface DragOrigin {
  pageNumber: number;
  pageElement: HTMLElement;
  hostRect: DOMRect;
  startX: number;
  startY: number;
}

const MIN_DRAG_PX = 16;

/** Supports Obsidian PDF++-style rectangular area capture via Area tool mode or Alt+Drag on any PDF page. */
export function usePdfAreaSelect(
  containerRef: Readonly<ShallowRef<HTMLDivElement | null>>,
  areaMode: Readonly<Ref<boolean>>,
  onCapture: (page: number, rect: PdfNormalizedRect) => void,
) {
  const marquee = shallowRef<AreaMarqueeBox>();
  let origin: DragOrigin | undefined;

  function onMouseMove(event: MouseEvent) {
    if (!origin) return;
    const x1 = Math.min(origin.startX, event.clientX);
    const y1 = Math.min(origin.startY, event.clientY);
    const x2 = Math.max(origin.startX, event.clientX);
    const y2 = Math.max(origin.startY, event.clientY);
    marquee.value = {
      left: x1 - origin.hostRect.left,
      top: y1 - origin.hostRect.top,
      width: x2 - x1,
      height: y2 - y1,
    };
  }

  function finishDrag(event: MouseEvent) {
    globalThis.removeEventListener("mousemove", onMouseMove);
    globalThis.removeEventListener("mouseup", finishDrag);
    const current = origin;
    origin = undefined;
    marquee.value = undefined;
    if (!current) return;
    const x1 = Math.min(current.startX, event.clientX);
    const y1 = Math.min(current.startY, event.clientY);
    const x2 = Math.max(current.startX, event.clientX);
    const y2 = Math.max(current.startY, event.clientY);
    if (x2 - x1 < MIN_DRAG_PX || y2 - y1 < MIN_DRAG_PX) return;
    const pageBox = current.pageElement.getBoundingClientRect();
    const normalized = normalizeBoxToPage(
      { left: x1, top: y1, right: x2, bottom: y2, width: x2 - x1, height: y2 - y1 },
      pageBox,
    );
    if (normalized) onCapture(current.pageNumber, normalized);
  }

  function onMouseDown(event: MouseEvent) {
    if (event.button !== 0 || !(areaMode.value || event.altKey)) return;
    const target = event.target instanceof Element ? event.target : null;
    const pageElement = target?.closest<HTMLElement>(".page[data-page-number]");
    const host = containerRef.value;
    if (!pageElement || !host) return;
    const pageNumber = Number(pageElement.dataset.pageNumber);
    if (!Number.isInteger(pageNumber) || pageNumber <= 0) return;
    event.preventDefault();
    event.stopPropagation();
    origin = {
      pageNumber,
      pageElement,
      hostRect: host.getBoundingClientRect(),
      startX: event.clientX,
      startY: event.clientY,
    };
    marquee.value = {
      left: event.clientX - origin.hostRect.left,
      top: event.clientY - origin.hostRect.top,
      width: 0,
      height: 0,
    };
    globalThis.addEventListener("mousemove", onMouseMove);
    globalThis.addEventListener("mouseup", finishDrag);
  }

  return { marquee, onMouseDown };
}
