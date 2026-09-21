import { nextTick, shallowRef, watch, type ComputedRef, type Ref } from "vue";

import type { StructureOutlineRow } from "../model/structure/outline-projection";

interface OutlineTreeNavigationOptions {
  expandedIds: Ref<ReadonlySet<string>>;
  rows: ComputedRef<readonly StructureOutlineRow[]>;
  root: Ref<HTMLElement | null>;
  toggle(id: string): void;
}

export function useOutlineTreeNavigation(options: OutlineTreeNavigationOptions) {
  const focusedId = shallowRef<string>();

  watch(
    options.rows,
    (rows) => {
      if (!rows.some(({ id }) => id === focusedId.value)) focusedId.value = rows[0]?.id;
    },
    { immediate: true },
  );

  function tabIndex(id: string, index: number) {
    return focusedId.value === id || (!focusedId.value && index === 0) ? 0 : -1;
  }

  function handleKeydown(event: KeyboardEvent, row: StructureOutlineRow, index: number) {
    const action = navigationAction(event.key, row, index);
    if (!action) return;
    event.preventDefault();
    event.stopPropagation();
    if (action.kind === "toggle") options.toggle(row.id);
    else if (action.targetId) focus(action.targetId);
  }

  function navigationAction(key: string, row: StructureOutlineRow, index: number) {
    const rows = options.rows.value;
    const focusTargets: Readonly<Record<string, string | undefined>> = {
      ArrowDown: rows[index + 1]?.id,
      ArrowUp: rows[index - 1]?.id,
      Home: rows[0]?.id,
      End: rows.at(-1)?.id,
    };
    if (key in focusTargets) return { kind: "focus" as const, targetId: focusTargets[key] };
    if (key === "ArrowRight") return rightArrowAction(row, index);
    if (key === "ArrowLeft") return leftArrowAction(row);
    return undefined;
  }

  function rightArrowAction(row: StructureOutlineRow, index: number) {
    if (!row.childCount) return undefined;
    return options.expandedIds.value.has(row.id)
      ? { kind: "focus" as const, targetId: options.rows.value[index + 1]?.id }
      : { kind: "toggle" as const };
  }

  function leftArrowAction(row: StructureOutlineRow) {
    return row.childCount && options.expandedIds.value.has(row.id)
      ? { kind: "toggle" as const }
      : { kind: "focus" as const, targetId: row.parentId ?? undefined };
  }

  function focus(id: string) {
    focusedId.value = id;
    void nextTick(() => {
      const rows = options.root.value?.querySelectorAll<HTMLElement>("[data-outline-tree-item]");
      [...(rows ?? [])].find((element) => element.dataset.outlineTreeItem === id)?.focus();
    });
  }

  function setFocusedId(id: string) {
    focusedId.value = id;
  }

  return { focus, handleKeydown, setFocusedId, tabIndex };
}
