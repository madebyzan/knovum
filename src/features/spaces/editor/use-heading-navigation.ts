import { nextTick, ref, watch } from "vue";

import { activeHeadingFromScroll, type spaceTiptapHeadings } from "./tiptap-outline";

function headingElement(
  editor: Parameters<typeof activeHeadingFromScroll>[0],
  position: number,
): HTMLElement | undefined {
  const node = editor?.view.nodeDOM(position);
  return node instanceof HTMLElement ? node : undefined;
}

export function useHeadingNavigation(
  editor: () => Parameters<typeof activeHeadingFromScroll>[0],
  headings: () => ReturnType<typeof spaceTiptapHeadings>,
) {
  const scrollRoot = ref<HTMLElement>();
  const activeHeadingId = ref<string>();
  let scrollFrame = 0;
  const syncActiveHeading = () => {
    globalThis.cancelAnimationFrame(scrollFrame);
    scrollFrame = globalThis.requestAnimationFrame(() => {
      activeHeadingId.value = activeHeadingFromScroll(editor(), headings(), scrollRoot.value);
    });
  };
  const revealHeading = (position: number) => {
    const root = scrollRoot.value;
    const target = headingElement(editor(), position);
    if (!root || !target) return;
    const top = root.scrollTop + target.getBoundingClientRect().top - root.getBoundingClientRect().top - 72;
    root.scrollTo({ top: Math.max(0, top), behavior: "smooth" });
  };
  watch(headings, () => void nextTick(syncActiveHeading));
  const dispose = () => globalThis.cancelAnimationFrame(scrollFrame);
  return { activeHeadingId, dispose, revealHeading, scrollRoot, syncActiveHeading };
}
