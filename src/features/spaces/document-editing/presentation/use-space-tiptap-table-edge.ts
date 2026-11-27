import type { Editor } from "@tiptap/vue-3";
import { onBeforeUnmount, shallowRef, watch } from "vue";

export interface TableEdgeBounds {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
  readonly lastCellPos: number;
}

const HOVER_BRIDGE_PX = 28;
const HIDE_DELAY_MS = 140;

function isSameBounds(a: TableEdgeBounds | null, b: TableEdgeBounds): boolean {
  return (
    a !== null &&
    a.left === b.left &&
    a.top === b.top &&
    a.width === b.width &&
    a.height === b.height &&
    a.lastCellPos === b.lastCellPos
  );
}

function isPointerNearBounds(event: PointerEvent, host: HTMLElement, current: TableEdgeBounds): boolean {
  const hostRect = host.getBoundingClientRect();
  const x = event.clientX - hostRect.left;
  const y = event.clientY - hostRect.top;
  return (
    x >= current.left - 8 &&
    x <= current.left + current.width + HOVER_BRIDGE_PX &&
    y >= current.top - 8 &&
    y <= current.top + current.height + HOVER_BRIDGE_PX
  );
}

function isInsideEdgeZone(
  event: PointerEvent,
  target: Element | null,
  host: HTMLElement | null,
  current: TableEdgeBounds | null,
): boolean {
  if (target?.closest("[data-table-edge-control]")) return true;
  return Boolean(host && current && isPointerNearBounds(event, host, current));
}

export function useSpaceTiptapTableEdge(getEditor: () => Editor, getHost: () => HTMLElement | null) {
  const box = shallowRef<TableEdgeBounds | null>(null);
  let rafId: number | null = null;
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  function cancelHide() {
    if (hideTimer !== null) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  }

  function scheduleHide() {
    if (hideTimer !== null) return;
    hideTimer = setTimeout(() => {
      hideTimer = null;
      if (!getEditor().isActive("table")) box.value = null;
    }, HIDE_DELAY_MS);
  }

  function findLastCellPos(tableEl: HTMLTableElement): number | null {
    const lastRow = tableEl.rows.item(tableEl.rows.length - 1);
    const lastCell = lastRow?.cells.item(lastRow.cells.length - 1);
    if (!lastCell) return null;
    try {
      return getEditor().view.posAtDOM(lastCell, 0);
    } catch {
      return null;
    }
  }

  function measureTable(tableEl: HTMLTableElement | null) {
    const host = getHost();
    const editor = getEditor();
    if (!host || !tableEl || !editor.isEditable) {
      box.value = null;
      return;
    }
    const lastCellPos = findLastCellPos(tableEl);
    if (lastCellPos === null) {
      box.value = null;
      return;
    }
    const hostRect = host.getBoundingClientRect();
    const wrapper = tableEl.closest(".tableWrapper") ?? tableEl;
    const rect = wrapper.getBoundingClientRect();
    const next: TableEdgeBounds = {
      left: Math.round(rect.left - hostRect.left),
      top: Math.round(rect.top - hostRect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
      lastCellPos,
    };
    if (!isSameBounds(box.value, next)) box.value = next;
  }

  function syncFromSelection() {
    const editor = getEditor();
    if (editor.isDestroyed || !editor.isActive("table")) return;
    try {
      const dom = editor.view.domAtPos(editor.state.selection.from).node;
      const el = dom instanceof Element ? dom : dom.parentElement;
      measureTable(el?.closest("table") ?? null);
    } catch {
      box.value = null;
    }
  }

  function scheduleRun(task: () => void) {
    if (rafId !== null) cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(() => {
      rafId = null;
      task();
    });
  }

  watch(
    () => [getHost(), getEditor()] as const,
    ([host, editor], _prev, cleanup) => {
      const onPointerMove = (event: PointerEvent) => {
        if (event.buttons !== 0) return;
        const target = event.target instanceof Element ? event.target : null;
        const table = target?.closest(".tableWrapper table") as HTMLTableElement | null;
        if (table) {
          cancelHide();
          scheduleRun(() => measureTable(table));
          return;
        }
        if (isInsideEdgeZone(event, target, host, box.value)) cancelHide();
        else if (!editor.isActive("table")) scheduleHide();
      };
      const onLeave = () => {
        if (!editor.isActive("table")) scheduleHide();
      };
      const onTransaction = ({
        transaction,
      }: {
        transaction: { docChanged: boolean; selectionSet: boolean };
      }) => {
        if (!transaction.docChanged && !transaction.selectionSet) return;
        if (editor.isActive("table")) {
          cancelHide();
          scheduleRun(syncFromSelection);
        } else if (box.value) {
          box.value = null;
        }
      };
      host?.addEventListener("pointermove", onPointerMove, { passive: true });
      host?.addEventListener("pointerleave", onLeave, { passive: true });
      editor.on("transaction", onTransaction);
      cleanup(() => {
        host?.removeEventListener("pointermove", onPointerMove);
        host?.removeEventListener("pointerleave", onLeave);
        editor.off("transaction", onTransaction);
      });
    },
    { immediate: true },
  );

  onBeforeUnmount(() => {
    cancelHide();
    if (rafId !== null) cancelAnimationFrame(rafId);
  });

  function appendRow() {
    if (!box.value) return;
    getEditor().chain().focus().setTextSelection(box.value.lastCellPos).addRowAfter().run();
    scheduleRun(syncFromSelection);
  }

  function appendColumn() {
    if (!box.value) return;
    getEditor().chain().focus().setTextSelection(box.value.lastCellPos).addColumnAfter().run();
    scheduleRun(syncFromSelection);
  }

  return { box, cancelHide, appendRow, appendColumn };
}
