import { computed, inject, shallowRef, watch } from "vue";

import { workspaceSidebarKey } from "../../../../shared/contracts/workspace-sidebar";
import { useCanvasSelectionProjection } from "../../canvas/use-canvas-selection-projection";
import { useSpaceStore } from "../../session/space-store";
import { classifyReadableSource } from "../source-reader-target";

const NARROW_VIEWPORT_WIDTH = 1120;

/**
 * Coordinates the Source Reader target, lazy read-surface mounting, history navigation,
 * follow-canvas-selection mode, and narrow-viewport sidebar auto-collapse.
 */
export function useSourceReaderState() {
  const store = useSpaceStore();
  const workspaceSidebar = inject(workspaceSidebarKey, undefined);
  const { selectedNodes } = useCanvasSelectionProjection(() => store.nodes);

  const target = computed(() => store.readingTarget);
  const sourceNode = computed(() =>
    target.value ? store.nodes.find((node) => node.id === target.value?.nodeId && !node.hidden) : undefined,
  );
  const readerKind = computed(() =>
    sourceNode.value ? classifyReadableSource(sourceNode.value.data) : undefined,
  );
  const activePane = computed(() => (readerKind.value ? (target.value?.pane ?? "read") : "info"));
  const readMounted = shallowRef(activePane.value === "read");
  const targetQuote = computed(() => target.value?.quote ?? target.value?.locator?.quote?.exact);
  const canBack = computed(() => store.readingHistoryIndex > 0);
  const canForward = computed(() => store.readingHistoryIndex < store.readingHistory.length - 1);

  watch(
    [target, sourceNode],
    ([currentTarget, currentNode]) => {
      if (currentTarget && !currentNode) store.closeSourceReader();
    },
    { flush: "post" },
  );

  watch(
    () => target.value?.nonce,
    (nonce) => {
      if (
        nonce !== undefined &&
        workspaceSidebar?.pinned.value &&
        globalThis.innerWidth < NARROW_VIEWPORT_WIDTH
      )
        workspaceSidebar.collapse();
    },
    { immediate: true },
  );

  watch([() => sourceNode.value?.id, activePane], ([id, pane], prev) => {
    if (id !== prev?.[0]) readMounted.value = pane === "read";
    else if (pane === "read") readMounted.value = true;
  });

  watch(selectedNodes, (nodes) => {
    if (!store.readingFollowsSelection || !target.value || nodes.length !== 1) return;
    const candidate = nodes[0]!;
    if (candidate.id === sourceNode.value?.id) return;
    store.openSourceReader({
      nodeId: candidate.id,
      pane: classifyReadableSource(candidate.data) ? "read" : "info",
    });
  });

  return {
    store,
    target,
    sourceNode,
    readerKind,
    activePane,
    readMounted,
    targetQuote,
    canBack,
    canForward,
  };
}
