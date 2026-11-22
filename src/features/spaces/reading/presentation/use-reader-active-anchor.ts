import { useEventListener } from "@vueuse/core";
import { computed, nextTick, onBeforeUnmount, shallowRef, toValue, watch, type MaybeRefOrGetter } from "vue";

const SCROLL_TOP_OFFSET = 24;
const NEAR_BOTTOM_PX = 8;

interface ReaderActiveAnchorOptions {
  scroller: MaybeRefOrGetter<HTMLElement | null | undefined>;
  selector: string;
  keyOf: (el: HTMLElement, index: number) => string | undefined;
  validIds: MaybeRefOrGetter<readonly string[]>;
  sourceKey: MaybeRefOrGetter<unknown>;
  enabled?: MaybeRefOrGetter<boolean>;
}

function isNearBottom(host: HTMLElement): boolean {
  return (
    host.scrollHeight > host.clientHeight &&
    host.scrollHeight - host.scrollTop - host.clientHeight < NEAR_BOTTOM_PX
  );
}

function resolveActiveId(
  host: HTMLElement,
  valid: readonly string[],
  validSet: ReadonlySet<string>,
  options: Pick<ReaderActiveAnchorOptions, "selector" | "keyOf">,
): string | undefined {
  if (isNearBottom(host)) return valid.at(-1);
  const activationLine = host.getBoundingClientRect().top + Math.min(128, host.clientHeight * 0.22);
  const elements = host.querySelectorAll<HTMLElement>(options.selector);
  let current: string | undefined;
  for (let index = 0; index < elements.length; index += 1) {
    const el = elements[index]!;
    const key = options.keyOf(el, index);
    if (key && validSet.has(key) && el.getBoundingClientRect().top <= activationLine) current = key;
  }
  return current ?? valid[0];
}

/**
 * Tracks the active heading inside a reading scroller and provides smooth, container-scoped
 * navigation. Programmatic jumps pin the chosen item immediately so clicking near the bottom of a
 * document never snaps back to an earlier heading; manual scrolling releases the pin seamlessly.
 */
export function useReaderActiveAnchor(options: ReaderActiveAnchorOptions) {
  const activeId = shallowRef<string>();
  const validList = computed(() => toValue(options.validIds));
  const validSet = computed(() => new Set(validList.value));
  let pinnedId: string | undefined;
  let frameId: number | undefined;

  function sync() {
    frameId = undefined;
    if (pinnedId !== undefined || (options.enabled !== undefined && !toValue(options.enabled))) return;
    const host = toValue(options.scroller);
    const valid = validList.value;
    const resolved = host && valid.length ? resolveActiveId(host, valid, validSet.value, options) : undefined;
    if (activeId.value !== resolved) activeId.value = resolved;
  }

  function scheduleSync() {
    if (pinnedId !== undefined || frameId !== undefined) return;
    frameId = requestAnimationFrame(sync);
  }

  function releasePin() {
    if (pinnedId === undefined) return;
    pinnedId = undefined;
    scheduleSync();
  }

  function scrollTo(id: string, element: HTMLElement | null | undefined) {
    const host = toValue(options.scroller);
    pinnedId = id;
    activeId.value = id;
    if (!host || !element) return;
    const offset = element.getBoundingClientRect().top - host.getBoundingClientRect().top;
    const top = Math.max(0, host.scrollTop + offset - SCROLL_TOP_OFFSET);
    host.scrollTo({ top, behavior: "smooth" });
  }

  const scrollerGetter = () => toValue(options.scroller);
  useEventListener(scrollerGetter, "scroll", scheduleSync, { passive: true });
  useEventListener(scrollerGetter, "wheel", releasePin, { passive: true });
  useEventListener(scrollerGetter, "pointerdown", releasePin, { passive: true });
  useEventListener(scrollerGetter, "touchstart", releasePin, { passive: true });

  watch(
    [scrollerGetter, () => toValue(options.sourceKey)],
    async () => {
      pinnedId = undefined;
      await nextTick();
      sync();
    },
    { immediate: true, flush: "post" },
  );

  onBeforeUnmount(() => {
    if (frameId !== undefined) cancelAnimationFrame(frameId);
  });

  return { activeId, scrollTo, sync };
}
