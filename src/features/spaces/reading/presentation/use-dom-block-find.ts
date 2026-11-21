import { watchDebounced } from "@vueuse/core";
import { ref, shallowRef, type Ref } from "vue";

const DEFAULT_BLOCK_SELECTOR = "p, li, td, h1, h2, h3, h4, blockquote, pre";

/** Shared in-document Cmd/Ctrl+F block finder for prose, Word, Website Reader, and Document views. */
export function useDomBlockFind(
  host: Readonly<Ref<HTMLElement | null>>,
  options?: {
    enabled?: () => boolean;
    selector?: string;
  },
) {
  const open = shallowRef(false);
  const query = ref("");
  const matched = shallowRef<readonly HTMLElement[]>([]);
  const matchIndex = shallowRef(0);
  const selector = options?.selector ?? DEFAULT_BLOCK_SELECTOR;

  function clearHighlight() {
    for (const el of matched.value) el.removeAttribute("data-reader-find-active");
    matched.value = [];
    matchIndex.value = 0;
  }

  function focusMatch(nextIndex: number) {
    const list = matched.value;
    if (!list.length) return;
    for (const el of list) el.removeAttribute("data-reader-find-active");
    const normalized = ((nextIndex % list.length) + list.length) % list.length;
    matchIndex.value = normalized;
    const active = list[normalized]!;
    active.setAttribute("data-reader-find-active", "");
    active.scrollIntoView({ behavior: "smooth", block: "center" });
  }

  function runFind(text: string) {
    clearHighlight();
    const needle = text.trim().toLowerCase();
    const container = host.value;
    if (!needle || !container) return;
    const hits = [...container.querySelectorAll<HTMLElement>(selector)].filter((el) =>
      el.textContent?.toLowerCase().includes(needle),
    );
    matched.value = hits;
    if (hits.length) focusMatch(0);
  }

  function reset() {
    open.value = false;
    query.value = "";
    clearHighlight();
  }

  function toggle() {
    if (open.value) reset();
    else open.value = true;
  }

  function onKeyDown(event: KeyboardEvent): boolean {
    if (options?.enabled && !options.enabled()) return false;
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
      event.preventDefault();
      open.value = true;
      return true;
    }
    if (event.key === "Escape" && open.value) {
      reset();
      return true;
    }
    return false;
  }

  watchDebounced(
    query,
    (value) => {
      if (open.value) runFind(value);
    },
    { debounce: 90 },
  );

  return {
    open,
    query,
    matchIndex,
    matched,
    runFind,
    clearHighlight,
    reset,
    toggle,
    next: () => focusMatch(matchIndex.value + 1),
    prev: () => focusMatch(matchIndex.value - 1),
    onKeyDown,
  };
}
