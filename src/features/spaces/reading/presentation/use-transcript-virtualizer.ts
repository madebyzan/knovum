import { useVirtualizer } from "@tanstack/vue-virtual";
import { computed, watch, type ComponentPublicInstance, type ComputedRef, type Ref } from "vue";

import type { SpaceTranscriptSegment } from "../../media/schema/transcript";

export function useTranscriptVirtualizer(
  scroller: Readonly<Ref<HTMLDivElement | null>>,
  segments: ComputedRef<readonly SpaceTranscriptSegment[]>,
) {
  const virtualizer = useVirtualizer(
    computed(() => ({
      count: segments.value.length,
      getScrollElement: () => scroller.value,
      getItemKey: (index: number) => segments.value[index]?.id ?? index,
      estimateSize: () => 52,
      overscan: 8,
      initialRect: { width: 600, height: 800 },
    })),
  );
  const rows = computed(() => virtualizer.value.getVirtualItems());
  const totalSize = computed(() => virtualizer.value.getTotalSize());

  watch(segments, () => virtualizer.value.measure(), { flush: "post" });

  function measureElement(element: Element | ComponentPublicInstance | null) {
    const row = element instanceof Element ? element : (element?.$el ?? null);
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

  return { rows, totalSize, measureElement, scrollToSegment };
}
