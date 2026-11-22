import { onBeforeUnmount, shallowRef, type CSSProperties, type Ref } from "vue";

import type { GroundedCitation } from "../../knowledge/model/schema/grounded-citation";

export const CITATION_HOVER_OPEN_DELAY_MS = 320;
export const CITATION_HOVER_CLOSE_DELAY_MS = 180;
const PREVIEW_WIDTH = 304;
const VIEWPORT_MARGIN = 12;
const ANCHOR_GAP = 6;
const TOP_FLIP_THRESHOLD = 168;

export interface CitationPreviewState {
  citation: GroundedCitation;
  placement: "top" | "bottom";
  style: CSSProperties;
}

function positionForRect(rect: DOMRect): { placement: "top" | "bottom"; style: CSSProperties } {
  const viewportWidth = globalThis.innerWidth || 1280;
  const viewportHeight = globalThis.innerHeight || 800;
  const centerX = rect.left + rect.width / 2;
  const left = Math.min(
    Math.max(VIEWPORT_MARGIN, centerX - PREVIEW_WIDTH / 2),
    Math.max(VIEWPORT_MARGIN, viewportWidth - PREVIEW_WIDTH - VIEWPORT_MARGIN),
  );
  if (rect.top < TOP_FLIP_THRESHOLD) {
    return {
      placement: "bottom",
      style: { left: `${Math.round(left)}px`, top: `${Math.round(rect.bottom + ANCHOR_GAP)}px` },
    };
  }
  return {
    placement: "top",
    style: {
      left: `${Math.round(left)}px`,
      bottom: `${Math.round(viewportHeight - rect.top + ANCHOR_GAP)}px`,
    },
  };
}

function citationTriggerFrom(target: EventTarget | null): HTMLElement | null {
  return target instanceof Element ? target.closest<HTMLElement>("[data-citation-index]") : null;
}

/**
 * Manages a delayed, pointer-bridged preview for inline `[data-citation-index]` pills inside
 * streamed Markdown answers. Global dismiss listeners stay detached while no preview is open.
 */
export function useCitationHover(citations: Ref<readonly GroundedCitation[]>) {
  const active = shallowRef<CitationPreviewState>();
  let openTimer: ReturnType<typeof globalThis.setTimeout> | undefined;
  let closeTimer: ReturnType<typeof globalThis.setTimeout> | undefined;
  let pendingTrigger: HTMLElement | undefined;
  let listenersAttached = false;

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") dismiss();
  }

  function onViewportChange() {
    dismiss();
  }

  function attachDismissListeners() {
    if (listenersAttached) return;
    listenersAttached = true;
    globalThis.addEventListener("keydown", onKeyDown);
    globalThis.addEventListener("scroll", onViewportChange, true);
    globalThis.addEventListener("resize", onViewportChange);
  }

  function detachDismissListeners() {
    if (!listenersAttached) return;
    listenersAttached = false;
    globalThis.removeEventListener("keydown", onKeyDown);
    globalThis.removeEventListener("scroll", onViewportChange, true);
    globalThis.removeEventListener("resize", onViewportChange);
  }

  function clearTimers() {
    if (openTimer !== undefined) {
      globalThis.clearTimeout(openTimer);
      openTimer = undefined;
    }
    if (closeTimer !== undefined) {
      globalThis.clearTimeout(closeTimer);
      closeTimer = undefined;
    }
  }

  function dismiss() {
    clearTimers();
    pendingTrigger = undefined;
    active.value = undefined;
    detachDismissListeners();
  }

  function showForTrigger(trigger: HTMLElement) {
    const index = Number(trigger.getAttribute("data-citation-index"));
    const citation = citations.value.find((item) => item.index === index);
    if (!citation) return;
    const { placement, style } = positionForRect(trigger.getBoundingClientRect());
    active.value = { citation, placement, style };
    attachDismissListeners();
  }

  function scheduleOpen(trigger: HTMLElement) {
    if (closeTimer !== undefined) {
      globalThis.clearTimeout(closeTimer);
      closeTimer = undefined;
    }
    const index = Number(trigger.getAttribute("data-citation-index"));
    if (active.value?.citation.index === index) {
      showForTrigger(trigger);
      return;
    }
    if (pendingTrigger === trigger && openTimer !== undefined) return;
    if (openTimer !== undefined) globalThis.clearTimeout(openTimer);
    pendingTrigger = trigger;
    const delay = active.value ? 120 : CITATION_HOVER_OPEN_DELAY_MS;
    openTimer = globalThis.setTimeout(() => {
      openTimer = undefined;
      if (pendingTrigger !== trigger || !trigger.isConnected) return;
      showForTrigger(trigger);
    }, delay);
  }

  function scheduleClose() {
    if (openTimer !== undefined) {
      globalThis.clearTimeout(openTimer);
      openTimer = undefined;
    }
    pendingTrigger = undefined;
    if (!active.value) return;
    if (closeTimer !== undefined) globalThis.clearTimeout(closeTimer);
    closeTimer = globalThis.setTimeout(() => {
      closeTimer = undefined;
      active.value = undefined;
      detachDismissListeners();
    }, CITATION_HOVER_CLOSE_DELAY_MS);
  }

  function onBodyPointerOver(event: PointerEvent) {
    const trigger = citationTriggerFrom(event.target);
    if (trigger) scheduleOpen(trigger);
  }

  function onBodyPointerOut(event: PointerEvent) {
    const trigger = citationTriggerFrom(event.target);
    if (!trigger) return;
    if (citationTriggerFrom(event.relatedTarget) === trigger) return;
    scheduleClose();
  }

  function onPreviewPointerEnter() {
    if (closeTimer !== undefined) {
      globalThis.clearTimeout(closeTimer);
      closeTimer = undefined;
    }
  }

  onBeforeUnmount(dismiss);

  return {
    active,
    dismiss,
    onBodyPointerOver,
    onBodyPointerOut,
    onPreviewPointerEnter,
    onPreviewPointerLeave: scheduleClose,
  };
}
