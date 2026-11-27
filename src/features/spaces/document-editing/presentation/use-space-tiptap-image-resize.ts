import type { Editor } from "@tiptap/core";
import { computed, onBeforeUnmount, shallowRef, watch, type ShallowRef } from "vue";

export const IMAGE_WIDTH_PRESETS = [
  ["50%", 0.5],
  ["75%", 0.75],
  ["100%", 1],
] as const;

export const IMAGE_RESIZE_SIDES = [
  ["left-2.5", -1],
  ["right-2.5", 1],
] as const;

const MIN_IMAGE_WIDTH = 160;
const SNAP_THRESHOLD_PX = 6;

interface ImageResizeOptions {
  readonly frameRef: Readonly<ShallowRef<HTMLElement | null>>;
  readonly badgeRef: Readonly<ShallowRef<HTMLElement | null>>;
  readonly imgRef: Readonly<ShallowRef<HTMLImageElement | null>>;
  readonly getEditor: () => Editor;
  readonly getWidthAttr: () => unknown;
  readonly updateSize: (width: number | null) => void;
  readonly onResizeEnd?: () => void;
}

interface ActiveDrag {
  readonly pointerId: number;
  readonly captureEl: HTMLElement;
  readonly startX: number;
  readonly startWidth: number;
  readonly maxWidth: number;
  readonly sign: 1 | -1;
  latestX: number;
  width: number;
  snappedFraction: number | null;
}

function resolveSnappedWidth(rawWidth: number, maxWidth: number): { width: number; fraction: number | null } {
  const clamped = Math.round(Math.min(maxWidth, Math.max(MIN_IMAGE_WIDTH, rawWidth)));
  for (const [, fraction] of IMAGE_WIDTH_PRESETS) {
    const target = Math.round(maxWidth * fraction);
    if (Math.abs(clamped - target) <= SNAP_THRESHOLD_PX) return { width: target, fraction };
  }
  return { width: clamped, fraction: null };
}

export function useSpaceTiptapImageResize(options: ImageResizeOptions) {
  const isEditable = shallowRef(options.getEditor().isEditable);
  let drag: ActiveDrag | null = null;
  let rafId: number | null = null;

  watch(
    options.getEditor,
    (editor, _prev, cleanup) => {
      const syncEditable = () => {
        isEditable.value = editor.isEditable;
      };
      syncEditable();
      editor.on("update", syncEditable);
      cleanup(() => editor.off("update", syncEditable));
    },
    { immediate: true },
  );

  const renderedWidth = computed(() => {
    const width = options.getWidthAttr();
    return typeof width === "number" && width > 0 ? `${width}px` : "";
  });

  watch(
    [renderedWidth, options.frameRef],
    ([width, frame]) => {
      if (!drag && frame) frame.style.width = width;
    },
    { immediate: true, flush: "post" },
  );

  function containerWidth(): number {
    return options.frameRef.value?.parentElement?.clientWidth ?? 720;
  }

  const activePreset = computed<number>(() => {
    const raw = options.getWidthAttr();
    const width = typeof raw === "number" && raw > 0 ? raw : null;
    if (!width) return 1;
    const ratio = width / Math.max(containerWidth(), 1);
    if (Math.abs(ratio - 0.5) < 0.06) return 0.5;
    if (Math.abs(ratio - 0.75) < 0.06) return 0.75;
    return ratio >= 0.94 ? 1 : 0;
  });

  function setPresetFraction(fraction: number) {
    if (fraction >= 1) {
      options.updateSize(null);
      return;
    }
    options.updateSize(Math.max(MIN_IMAGE_WIDTH, Math.round(containerWidth() * fraction)));
  }

  function paintFrame() {
    rafId = null;
    if (!drag) return;
    const delta = (drag.latestX - drag.startX) * drag.sign * 2;
    const snapped = resolveSnappedWidth(drag.startWidth + delta, drag.maxWidth);
    drag.width = snapped.width;
    drag.snappedFraction = snapped.fraction;
    const frame = options.frameRef.value;
    if (frame) frame.style.width = `${snapped.width}px`;
    const badge = options.badgeRef.value;
    if (badge) {
      badge.textContent =
        snapped.fraction !== null
          ? `${Math.round(snapped.fraction * 100)}% · ${snapped.width} px`
          : `${snapped.width} px`;
    }
  }

  function onPointerMove(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointerId) return;
    drag.latestX = event.clientX;
    if (rafId === null) rafId = requestAnimationFrame(paintFrame);
  }

  function releaseDragResources(current: ActiveDrag | null) {
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
    globalThis.removeEventListener("pointermove", onPointerMove);
    globalThis.removeEventListener("pointerup", onPointerUp);
    globalThis.removeEventListener("pointercancel", onPointerCancel);
    if (current?.captureEl.hasPointerCapture?.(current.pointerId)) {
      current.captureEl.releasePointerCapture(current.pointerId);
    }
    options.frameRef.value?.removeAttribute("data-resizing");
    if (typeof document !== "undefined") document.body.style.cursor = "";
  }

  function finishDrag(commit: boolean) {
    const current = drag;
    drag = null;
    releaseDragResources(current);
    if (commit && current) {
      const finalWidth = current.snappedFraction === 1 ? null : current.width;
      if (options.frameRef.value) options.frameRef.value.style.width = finalWidth ? `${finalWidth}px` : "";
      options.updateSize(finalWidth);
      options.onResizeEnd?.();
    } else if (options.frameRef.value) {
      options.frameRef.value.style.width = renderedWidth.value;
    }
  }

  function onPointerUp(event: PointerEvent) {
    if (drag && event.pointerId === drag.pointerId) finishDrag(true);
  }

  function onPointerCancel(event: PointerEvent) {
    if (drag && event.pointerId === drag.pointerId) finishDrag(false);
  }

  function measureStartBounds() {
    const maxWidth = containerWidth();
    const startWidth = Math.round(options.imgRef.value?.getBoundingClientRect().width ?? maxWidth);
    return { maxWidth, startWidth };
  }

  function startResize(event: PointerEvent, sign: 1 | -1) {
    if (!isEditable.value || event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const captureEl = event.currentTarget as HTMLElement;
    const { maxWidth, startWidth } = measureStartBounds();
    drag = {
      pointerId: event.pointerId,
      captureEl,
      startX: event.clientX,
      latestX: event.clientX,
      startWidth,
      maxWidth,
      sign,
      width: startWidth,
      snappedFraction: null,
    };
    captureEl.setPointerCapture?.(event.pointerId);
    options.frameRef.value?.setAttribute("data-resizing", "true");
    if (typeof document !== "undefined") document.body.style.cursor = "ew-resize";
    paintFrame();
    globalThis.addEventListener("pointermove", onPointerMove, { passive: true });
    globalThis.addEventListener("pointerup", onPointerUp);
    globalThis.addEventListener("pointercancel", onPointerCancel);
  }

  onBeforeUnmount(() => finishDrag(false));

  return { isEditable, renderedWidth, activePreset, setPresetFraction, startResize };
}
