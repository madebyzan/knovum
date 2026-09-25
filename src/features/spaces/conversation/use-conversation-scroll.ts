import { useVirtualizer } from "@tanstack/vue-virtual";
import {
  computed,
  nextTick,
  onBeforeUnmount,
  ref,
  shallowRef,
  watch,
  type ComponentPublicInstance,
} from "vue";

export function useConversationScroll(
  props: {
    messages: readonly { id: string }[];
    streamedText: string;
    loadingOlder: boolean;
    scrollKey: string;
    savedScrollTop: number;
  },
  events: { loadOlder: () => void; remember: (key: string, top: number) => void },
) {
  const scroller = ref<HTMLDivElement | null>(null);
  const atTail = shallowRef(true);
  let olderScrollHeight: number | undefined;
  let olderScrollTop = 0;
  let restorePending = true;
  const virtualizer = useVirtualizer(
    computed(() => ({
      count: props.messages.length,
      getScrollElement: () => scroller.value,
      estimateSize: () => 92,
      overscan: 6,
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

  function loadOlder() {
    const element = scroller.value;
    if (element) {
      olderScrollHeight = element.scrollHeight;
      olderScrollTop = element.scrollTop;
    }
    events.loadOlder();
  }

  watch(
    () => [props.messages.length, props.streamedText],
    async () => {
      if (olderScrollHeight !== undefined) {
        const previousHeight = olderScrollHeight;
        olderScrollHeight = undefined;
        await nextTick();
        const element = scroller.value;
        if (element) element.scrollTop = olderScrollTop + element.scrollHeight - previousHeight;
        return;
      }
      if (restorePending && props.messages.length) {
        await nextTick();
        const element = scroller.value;
        if (element) {
          element.scrollTop = props.savedScrollTop;
          updateScrollPosition();
          restorePending = false;
        }
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
      const element = scroller.value;
      if (element && props.messages.length) {
        element.scrollTop = props.savedScrollTop;
        updateScrollPosition();
        restorePending = false;
      }
    },
    { immediate: true },
  );
  onBeforeUnmount(() => {
    if (scroller.value) events.remember(props.scrollKey, scroller.value.scrollTop);
  });

  return { scroller, atTail, rows, totalSize, measureElement, updateScrollPosition, scrollToTail, loadOlder };
}
