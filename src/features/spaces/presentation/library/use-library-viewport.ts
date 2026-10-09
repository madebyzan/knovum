import { defaultRangeExtractor, useVirtualizer } from "@tanstack/vue-virtual";
import { useElementSize } from "@vueuse/core";
import { computed, nextTick, shallowRef, watch, type Ref } from "vue";

import type { Space } from "../../model/space";

export function useLibraryViewport(options: {
  spaces: Readonly<Ref<Space[]>>;
  layout: Readonly<Ref<"grid" | "list">>;
  root: Readonly<Ref<HTMLElement | null>>;
  host: Readonly<Ref<HTMLElement | null>>;
  scroll: () => HTMLElement | undefined;
  editingId: Readonly<Ref<string | undefined>>;
}) {
  const { width } = useElementSize(options.root);
  const columns = computed(() =>
    options.layout.value === "list" ? 1 : Math.max(1, Math.floor((width.value + 20) / 230)),
  );
  const margin = computed(() => options.host.value?.offsetTop ?? 0);
  const focused = shallowRef<string>();
  const focusedId = computed(() =>
    options.spaces.value.some(({ id }) => id === focused.value) ? focused.value : options.spaces.value[0]?.id,
  );
  const virtualizer = useVirtualizer(
    computed(() => {
      const retained = [focusedId.value, options.editingId.value].flatMap((id) => {
        const index = options.spaces.value.findIndex((space) => space.id === id);
        return index >= 0 ? [Math.floor(index / columns.value)] : [];
      });
      return {
        count: Math.ceil(options.spaces.value.length / columns.value),
        getScrollElement: () => options.scroll() ?? null,
        estimateSize: () =>
          options.layout.value === "list"
            ? 86
            : (Math.min(240, (width.value - (columns.value - 1) * 20) / columns.value) * 9) / 16 + 80,
        overscan: 3,
        scrollMargin: margin.value,
        rangeExtractor: (range) =>
          [...new Set([...defaultRangeExtractor(range), ...retained])].toSorted((a, b) => a - b),
      };
    }),
  );
  async function reveal(spaceId: string) {
    const index = options.spaces.value.findIndex((space) => space.id === spaceId);
    if (index < 0) return;
    focused.value = spaceId;
    virtualizer.value.scrollToIndex(Math.floor(index / columns.value), { align: "auto" });
    await nextTick();
  }
  function keydown(event: KeyboardEvent) {
    if (
      event.isComposing ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      !(event.target instanceof HTMLButtonElement) ||
      !event.target.dataset.librarySpaceId
    )
      return;
    const currentId = event.target.dataset.librarySpaceId;
    const current = options.spaces.value.findIndex((space) => space.id === currentId);
    const offsets: Record<string, number> = {
      ArrowLeft: -1,
      ArrowRight: 1,
      ArrowUp: -columns.value,
      ArrowDown: columns.value,
      Home: -current,
      End: options.spaces.value.length - 1 - current,
    };
    const offset = offsets[event.key];
    if (offset === undefined) return;
    event.preventDefault();
    const target =
      options.spaces.value[Math.max(0, Math.min(options.spaces.value.length - 1, current + offset))];
    if (target)
      void reveal(target.id).then(() =>
        options.root.value
          ?.querySelector<HTMLButtonElement>(`[data-library-space-id="${CSS.escape(target.id)}"]`)
          ?.focus({ preventScroll: true }),
      );
  }
  watch([columns, options.layout], () => virtualizer.value.measure());
  watch(options.spaces, () => {
    if (!options.spaces.value.some(({ id }) => id === focused.value)) focused.value = undefined;
  });
  return {
    measure: (element: HTMLElement | null) => {
      if (element) virtualizer.value.measureElement(element);
    },
    focus: (spaceId: string) => {
      focused.value = spaceId;
    },
    reset: () => virtualizer.value.scrollToOffset(0),
    columns,
    focusedId,
    margin,
    rows: computed(() => virtualizer.value.getVirtualItems()),
    height: computed(() => virtualizer.value.getTotalSize()),
    reveal,
    keydown,
  };
}
