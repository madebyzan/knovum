import { useVirtualizer } from "@tanstack/vue-virtual";
import {
  computed,
  nextTick,
  onBeforeUnmount,
  shallowRef,
  watch,
  type ComponentPublicInstance,
  type ShallowRef,
} from "vue";

interface ConversationScrollOptions {
  rows: readonly { id: string }[];
  streamedText: string;
  loadingOlder: boolean;
  scrollKey: string;
  savedScrollTop?: number | undefined;
}

/** A guess used only until a turn has been measured; every row reports its real height. */
const ESTIMATED_TURN_HEIGHT = 140;

export function useConversationScroll(
  scroller: Readonly<ShallowRef<HTMLDivElement | null>>,
  props: ConversationScrollOptions,
  events: { loadOlder: () => void; remember: (key: string, top: number) => void },
) {
  const atTail = shallowRef(true);
  let olderScrollHeight: number | undefined;
  let olderScrollTop = 0;
  let restorePending = true;
  const virtualizer = useVirtualizer(
    computed(() => ({
      count: props.rows.length,
      getScrollElement: () => scroller.value,
      getItemKey: (index) => props.rows[index]!.id,
      estimateSize: () => ESTIMATED_TURN_HEIGHT,
      overscan: 4,
    })),
  );
  const rows = computed(() => virtualizer.value.getVirtualItems());
  const totalSize = computed(() => virtualizer.value.getTotalSize());

  function measureElement(element: Element | ComponentPublicInstance | null) {
    const row = element instanceof Element ? element : element?.$el;
    if (row instanceof Element) virtualizer.value.measureElement(row);
  }

  function updateScrollPosition() {
    const element = scroller.value;
    if (!element) return;
    atTail.value = element.scrollHeight - element.scrollTop - element.clientHeight < 48;
  }

  function scrollToTail() {
    const element = scroller.value;
    element?.scrollTo({ top: element.scrollHeight });
    atTail.value = true;
  }

  function restoreScrollPosition() {
    const element = scroller.value;
    if (!element) return;
    if (props.savedScrollTop === undefined) scrollToTail();
    else {
      element.scrollTop = props.savedScrollTop;
      updateScrollPosition();
    }
    restorePending = false;
  }

  function loadOlder() {
    const element = scroller.value;
    if (element) {
      olderScrollHeight = element.scrollHeight;
      olderScrollTop = element.scrollTop;
    }
    events.loadOlder();
  }

  watch(
    () => [props.rows.length, props.streamedText],
    async () => {
      if (olderScrollHeight !== undefined) {
        const previousHeight = olderScrollHeight;
        olderScrollHeight = undefined;
        await nextTick();
        const element = scroller.value;
        if (element) element.scrollTop = olderScrollTop + element.scrollHeight - previousHeight;
        return;
      }
      if (restorePending && props.rows.length) {
        await nextTick();
        restoreScrollPosition();
        return;
      }
      if (!atTail.value) return;
      await nextTick();
      scrollToTail();
    },
    { immediate: true },
  );
  watch(
    () => props.loadingOlder,
    async (loading) => {
      if (!loading) {
        await nextTick();
        olderScrollHeight = undefined;
      }
    },
  );
  watch(
    () => props.scrollKey,
    async (_, previousKey) => {
      if (previousKey !== undefined && scroller.value) events.remember(previousKey, scroller.value.scrollTop);
      restorePending = true;
      await nextTick();
      if (props.rows.length) restoreScrollPosition();
    },
    { immediate: true },
  );
  onBeforeUnmount(() => {
    if (scroller.value) events.remember(props.scrollKey, scroller.value.scrollTop);
  });

  return { atTail, rows, totalSize, measureElement, updateScrollPosition, scrollToTail, loadOlder };
}
