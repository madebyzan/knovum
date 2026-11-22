import { useEventListener, useTextSelection } from "@vueuse/core";
import { computed, onBeforeUnmount, shallowRef, type ShallowRef } from "vue";

import type { KnowledgeLocation } from "../../knowledge/model/schema/chunk";
import { extractReaderRangeText, rangeCoversPoint } from "../reader-location-resolvers";

const EDGE_MARGIN = 8;
const TOP_CLEARANCE = 44;

export interface SourceReaderPassage {
  text: string;
  location: KnowledgeLocation;
}

export interface SourceReaderSelectionAnchor {
  left: number;
  top: number;
  placement: "top" | "bottom";
}

export interface SurfaceSelectionState {
  passage: SourceReaderPassage;
  anchor: SourceReaderSelectionAnchor;
  clear?: () => void;
}

/** Positions the floating selection bar above or below `box` within `frame`, or hides if off-screen. */
function anchorSelectionBar(
  box: Pick<DOMRect, "left" | "top" | "right" | "bottom" | "width" | "height">,
  frame: Pick<DOMRect, "left" | "top" | "bottom" | "width" | "height">,
): SourceReaderSelectionAnchor | undefined {
  if (box.width === 0 && box.height === 0) return undefined;
  if (box.bottom < frame.top + 8 || box.top > frame.bottom - 8) return undefined;
  const center = box.left - frame.left + box.width / 2;
  const left = Math.min(Math.max(center, EDGE_MARGIN), Math.max(frame.width - EDGE_MARGIN, EDGE_MARGIN));
  const relativeTop = box.top - frame.top;
  if (relativeTop >= TOP_CLEARANCE) {
    return { left, top: Math.max(8, relativeTop - 6), placement: "top" };
  }
  const relativeBottom = box.bottom - frame.top;
  return {
    left,
    top: Math.min(Math.max(frame.height - 40, 8), relativeBottom + 6),
    placement: "bottom",
  };
}

/** Anchors the floating selection bar to the union bounding box of two corner DOM elements. */
export function anchorElementRange(
  first: Element | null,
  last: Element | null,
  host: Element,
): SourceReaderSelectionAnchor | undefined {
  if (!first || !last) return undefined;
  const a = first.getBoundingClientRect();
  const z = last.getBoundingClientRect();
  const left = Math.min(a.left, z.left);
  const top = Math.min(a.top, z.top);
  const right = Math.max(a.right, z.right);
  const bottom = Math.max(a.bottom, z.bottom);
  return anchorSelectionBar(
    { left, top, right, bottom, width: right - left, height: bottom - top },
    host.getBoundingClientRect(),
  );
}

function isEditableSelectionNode(node: Node): boolean {
  const element = node instanceof Element ? node : node.parentElement;
  return Boolean(element?.closest("input, textarea, [contenteditable='true']"));
}

export function useReaderSelection(
  container: Readonly<ShallowRef<HTMLElement | null>>,
  resolveLocation: (range: Range, host: HTMLElement) => KnowledgeLocation | undefined,
) {
  const { selection, ranges } = useTextSelection();
  const clearRevision = shallowRef(0);
  const scrollRevision = shallowRef(0);
  let frameId: number | undefined;
  const scheduleLayoutBump = () => {
    if (!ranges.value.length || frameId !== undefined) return;
    frameId = requestAnimationFrame(() => {
      frameId = undefined;
      scrollRevision.value += 1;
    });
  };
  useEventListener(container, "scroll", scheduleLayoutBump, { capture: true, passive: true });
  useEventListener("resize", scheduleLayoutBump, { passive: true });
  onBeforeUnmount(() => {
    if (frameId !== undefined) cancelAnimationFrame(frameId);
  });

  const range = computed(() => {
    void clearRevision.value;
    const [candidate] = ranges.value;
    const host = container.value;
    if (!candidate || candidate.collapsed || !host) return undefined;
    if (!candidate.commonAncestorContainer.isConnected) return undefined;
    if (!host.contains(candidate.commonAncestorContainer)) return undefined;
    if (isEditableSelectionNode(candidate.commonAncestorContainer)) return undefined;
    return candidate;
  });

  const passage = computed<SourceReaderPassage | undefined>(() => {
    const activeRange = range.value;
    const host = container.value;
    if (!activeRange || !host) return undefined;
    const text = extractReaderRangeText(activeRange);
    if (!text) return undefined;
    const location = resolveLocation(activeRange, host);
    return location ? { text, location } : undefined;
  });

  const anchor = computed<SourceReaderSelectionAnchor | undefined>(() => {
    void scrollRevision.value;
    const activeRange = range.value;
    const host = container.value;
    if (!activeRange || !passage.value || !host) return undefined;
    return anchorSelectionBar(activeRange.getBoundingClientRect(), host.getBoundingClientRect());
  });

  function covers(point: { x: number; y: number }) {
    return rangeCoversPoint(range.value, point);
  }

  function onContextMenu(event: MouseEvent) {
    if (passage.value) event.preventDefault();
  }

  return {
    passage,
    anchor,
    covers,
    onContextMenu,
    clear: () => {
      selection.value?.removeAllRanges();
      clearRevision.value += 1;
    },
  };
}
