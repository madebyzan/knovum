import type { Editor } from "@tiptap/vue-3";
import { onBeforeUnmount, shallowRef, watch, type Ref } from "vue";

import { moveTopLevelBlockToIndex } from "../tiptap-block-actions";
import {
  hitTestHoveredBlock,
  measureTopLevelBlocks,
  resolveBlockDropSlot,
  type BlockDropSlot,
  type TopLevelBlockMetric,
} from "../tiptap-block-gutter";
import { setBlockDragSource, setBlockMenuHighlight } from "../tiptap-block-highlight";

export interface BlockDragGhost {
  readonly x: number;
  readonly y: number;
  readonly label: string;
}

const DRAG_THRESHOLD_PX = 4;
const EDGE_SCROLL_ZONE = 56;
const EDGE_SCROLL_STEP = 10;

function cancelRaf(id: number | null): null {
  if (id !== null) cancelAnimationFrame(id);
  return null;
}

export function useSpaceTiptapBlockDrag(options: {
  readonly editor: Readonly<Ref<Editor>>;
  readonly proseHost: Readonly<Ref<HTMLElement | null>>;
  readonly scrollRoot: Readonly<Ref<HTMLElement | null>>;
  readonly suppressed?: Readonly<Ref<boolean>>;
}) {
  const hovered = shallowRef<TopLevelBlockMetric | null>(null);
  const locked = shallowRef<TopLevelBlockMetric | null>(null);
  const menuOpen = shallowRef(false);
  const dragging = shallowRef(false);
  const dropSlot = shallowRef<BlockDropSlot | null>(null);
  const ghost = shallowRef<BlockDragGhost | null>(null);

  let dragStart: { x: number; y: number; block: TopLevelBlockMetric } | null = null;
  let scrollRafId: number | null = null;
  let hoverRafId: number | null = null;
  let pendingHoverY = 0;
  let latestClientY = 0;

  function updateDragTarget(clientX: number, clientY: number) {
    latestClientY = clientY;
    const host = options.proseHost.value;
    if (!host || !dragStart) return;
    const yInHost = clientY - host.getBoundingClientRect().top;
    dropSlot.value = resolveBlockDropSlot(measureTopLevelBlocks(options.editor.value, host), yInHost);
    ghost.value = { x: clientX + 14, y: clientY - 12, label: dragStart.block.label };
  }

  function stepAutoScroll() {
    const root = options.scrollRoot.value;
    if (!dragging.value || !root) {
      scrollRafId = null;
      return;
    }
    const rect = root.getBoundingClientRect();
    if (latestClientY < rect.top + EDGE_SCROLL_ZONE) root.scrollTop -= EDGE_SCROLL_STEP;
    else if (latestClientY > rect.bottom - EDGE_SCROLL_ZONE) root.scrollTop += EDGE_SCROLL_STEP;
    if (ghost.value) updateDragTarget(ghost.value.x - 14, latestClientY);
    scrollRafId = requestAnimationFrame(stepAutoScroll);
  }

  function clearDrag() {
    scrollRafId = cancelRaf(scrollRafId);
    globalThis.removeEventListener("pointermove", onWindowPointerMove);
    globalThis.removeEventListener("pointerup", onWindowPointerUp);
    globalThis.removeEventListener("pointercancel", clearDrag);
    globalThis.removeEventListener("keydown", handleEscape);
    dragStart = null;
    dragging.value = false;
    dropSlot.value = null;
    ghost.value = null;
    setBlockDragSource(options.editor.value, null);
  }

  function handleEscape(event: KeyboardEvent) {
    if (event.key === "Escape" && dragging.value) {
      event.preventDefault();
      clearDrag();
    }
  }

  function onWindowPointerMove(event: PointerEvent) {
    if (!dragStart) return;
    const moved = Math.hypot(event.clientX - dragStart.x, event.clientY - dragStart.y);
    if (!dragging.value && moved >= DRAG_THRESHOLD_PX) {
      dragging.value = true;
      menuOpen.value = false;
      setBlockDragSource(options.editor.value, dragStart.block.pos);
      scrollRafId = requestAnimationFrame(stepAutoScroll);
    }
    if (dragging.value) updateDragTarget(event.clientX, event.clientY);
  }

  function onWindowPointerUp() {
    if (!dragStart) return;
    const { block } = dragStart;
    if (dragging.value) {
      const slot = dropSlot.value;
      clearDrag();
      if (slot) moveTopLevelBlockToIndex(options.editor.value, block.index, slot.targetIndex);
      hovered.value = null;
      return;
    }
    clearDrag();
    locked.value = block;
    menuOpen.value = true;
  }

  function onHandlePointerDown(event: PointerEvent) {
    if (event.button !== 0 || !hovered.value || !options.editor.value.isEditable) return;
    event.preventDefault();
    hoverRafId = cancelRaf(hoverRafId);
    dragStart = { x: event.clientX, y: event.clientY, block: hovered.value };
    globalThis.addEventListener("pointermove", onWindowPointerMove, { passive: true });
    globalThis.addEventListener("pointerup", onWindowPointerUp);
    globalThis.addEventListener("pointercancel", clearDrag);
    globalThis.addEventListener("keydown", handleEscape);
  }

  function isGutterSuppressed() {
    const ed = options.editor.value;
    return Boolean(
      options.suppressed?.value ||
      menuOpen.value ||
      dragging.value ||
      !ed.isEditable ||
      !ed.state.selection.empty,
    );
  }

  function hideHovered() {
    hoverRafId = cancelRaf(hoverRafId);
    if (!menuOpen.value && !dragging.value) hovered.value = null;
  }

  function onHostPointerMove(event: PointerEvent) {
    if (isGutterSuppressed()) return hideHovered();
    pendingHoverY = event.clientY;
    if (hoverRafId !== null) return;
    hoverRafId = requestAnimationFrame(() => {
      hoverRafId = null;
      if (isGutterSuppressed()) return hideHovered();
      const host = options.proseHost.value;
      if (!host) return;
      const rect = host.getBoundingClientRect();
      const yInHost = pendingHoverY - rect.top;
      hovered.value =
        yInHost < -4 || yInHost > rect.height + 4
          ? null
          : hitTestHoveredBlock(measureTopLevelBlocks(options.editor.value, host), yInHost);
    });
  }

  watch(
    () => options.editor.value,
    (instance, _prev, cleanup) => {
      const onSelection = () => {
        if (!instance.state.selection.empty) hideHovered();
      };
      instance.on("selectionUpdate", onSelection);
      cleanup(() => instance.off("selectionUpdate", onSelection));
    },
    { immediate: true },
  );
  watch(
    () => options.suppressed?.value,
    (suppressed) => suppressed && hideHovered(),
  );
  watch(menuOpen, (open) => {
    setBlockMenuHighlight(options.editor.value, open ? (locked.value?.pos ?? null) : null);
    if (!open) locked.value = null;
  });
  onBeforeUnmount(() => {
    hoverRafId = cancelRaf(hoverRafId);
    clearDrag();
    setBlockMenuHighlight(options.editor.value, null);
  });

  return {
    hovered,
    locked,
    menuOpen,
    dragging,
    dropSlot,
    ghost,
    onHandlePointerDown,
    onHostPointerMove,
    hideHovered,
  };
}
