import type { CanvasNodePresentationContext } from "../../../canvas/preview/policy";

export function projectPreviewItems<T>(
  items: readonly T[],
  context: CanvasNodePresentationContext,
): readonly T[] {
  return context.mode === "interactive" ? items : items.slice(0, context.viewport.previewItemLimit);
}

export function remainingPreviewItemCount(total: number, visible: number) {
  return Math.max(0, total - visible);
}
