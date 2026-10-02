import { nextTick, ref, watch } from "vue";

interface OutlineNode {
  type: { name: string };
  attrs: Record<string, unknown>;
  textContent: string;
}

interface OutlineEditor {
  state: { doc: { descendants: (visitor: (node: OutlineNode, position: number) => void) => void } };
  view: { nodeDOM: (position: number) => Node | null };
}

export interface SpaceTiptapHeading {
  id: string;
  level: 1 | 2 | 3 | 4 | 5 | 6;
  text: string;
  position: number;
}

export function spaceTiptapHeadings(editor?: OutlineEditor): SpaceTiptapHeading[] {
  if (!editor) return [];
  const headings: SpaceTiptapHeading[] = [];
  editor.state.doc.descendants((node, position) => {
    if (node.type.name !== "heading") return;
    const level = Number(node.attrs.level);
    if (level < 1 || level > 6) return;
    headings.push({
      id: `heading-${position}`,
      level: level as 1 | 2 | 3 | 4 | 5 | 6,
      text: node.textContent || "Untitled",
      position,
    });
  });
  return headings;
}

export function activeHeadingFromScroll(
  editor: OutlineEditor | undefined,
  headings: readonly SpaceTiptapHeading[],
  scrollRoot: HTMLElement | undefined,
): string | undefined {
  if (!editor || !scrollRoot || headings.length === 0) return undefined;
  const rootTop = scrollRoot.getBoundingClientRect().top;
  const activationLine = rootTop + Math.min(128, scrollRoot.clientHeight * 0.22);
  const active = lastHeadingAbove(editor, headings, activationLine);
  if (isNearScrollBottom(scrollRoot)) return headings.at(-1)?.id;
  return active?.id ?? headings[0]?.id;
}

function lastHeadingAbove(
  editor: OutlineEditor,
  headings: readonly SpaceTiptapHeading[],
  activationLine: number,
) {
  let active: SpaceTiptapHeading | undefined;
  for (const heading of headings) {
    const node = editor.view.nodeDOM(heading.position);
    if (!(node instanceof HTMLElement)) continue;
    if (node.getBoundingClientRect().top > activationLine) break;
    active = heading;
  }
  return active;
}

function isNearScrollBottom(scrollRoot: HTMLElement) {
  return scrollRoot.scrollHeight - scrollRoot.scrollTop - scrollRoot.clientHeight < 8;
}

export function headingElement(editor: OutlineEditor | undefined, position: number): HTMLElement | undefined {
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
