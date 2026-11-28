import { observeElementRect, useVirtualizer, type Rect, type Virtualizer } from "@tanstack/vue-virtual";
import { computed, watch, type ComponentPublicInstance, type ComputedRef, type Ref } from "vue";

import type { SpaceTranscriptSegment } from "../../media/schema/transcript";

function observeNonEmptyElementRect(
  instance: Virtualizer<HTMLDivElement, Element>,
  cb: (rect: Rect) => void,
) {
  return observeElementRect(instance, (rect) => {
    if (rect.height > 0) cb(rect);
  });
}

export function useTranscriptVirtualizer(
  scroller: Readonly<Ref<HTMLDivElement | null>>,
  segments: ComputedRef<readonly SpaceTranscriptSegment[]>,
) {
  const virtualizer = useVirtualizer<HTMLDivElement, Element>(
    computed(() => ({
      count: segments.value.length,
      getScrollElement: () => scroller.value,
      getItemKey: (index: number) => segments.value[index]?.id ?? index,
      estimateSize: () => 52,
      overscan: 8,
      initialRect: { width: 600, height: 800 },
      observeElementRect: observeNonEmptyElementRect,
    })),
  );
  const rows = computed(() => virtualizer.value.getVirtualItems());
  const totalSize = computed(() => virtualizer.value.getTotalSize());
  const paddingTop = computed(() => rows.value[0]?.start ?? 0);
  const paddingBottom = computed(() => {
    const last = rows.value.at(-1);
    return last ? Math.max(0, totalSize.value - last.end) : 0;
  });

  function syncScrollOffset() {
    const element = scroller.value;
    const instance = virtualizer.value;
    if (!element) {
      if (instance.scrollOffset) instance.scrollOffset = 0;
      return;
    }
    if (element.scrollTop === 0 && instance.scrollOffset) {
      instance.scrollOffset = 0;
    }
  }

  function resetScroll() {
    const element = scroller.value;
    if (element) element.scrollTop = 0;
    const instance = virtualizer.value;
    instance.scrollOffset = 0;
    instance.measure();
  }

  function refreshLayout() {
    syncScrollOffset();
    virtualizer.value.measure();
  }

  watch(
    [segments, scroller],
    () => {
      syncScrollOffset();
      virtualizer.value.measure();
    },
    { flush: "post" },
  );

  function measureElement(element: Element | ComponentPublicInstance | null) {
    const row = element instanceof Element ? element : (element?.$el ?? null);
    if (row instanceof HTMLElement && row.offsetHeight === 0) return;
    virtualizer.value.measureElement(row instanceof Element ? row : null);
  }

  function scrollToSegment(startMs: number, block: ScrollLogicalPosition) {
    const index = segments.value.findIndex((segment) => segment.startMs === startMs);
    if (index < 0) return;
    virtualizer.value.scrollToIndex(index, {
      align: block === "center" ? "center" : "start",
      behavior: "smooth",
    });
  }

  return {
    rows,
    totalSize,
    paddingTop,
    paddingBottom,
    measureElement,
    scrollToSegment,
    resetScroll,
    refreshLayout,
  };
}
