import type { CanvasNodePresentationContext } from "../../../canvas/preview/detail-profile";
import type { TiptapJsonNode } from "../../../editor/tiptap-document";

export function projectTiptapPreview(
  document: TiptapJsonNode,
  context: CanvasNodePresentationContext,
): TiptapJsonNode {
  if (context.mode === "interactive" || !document.content) return document;
  return {
    ...document,
    content: document.content.slice(0, context.viewport.previewItemLimit),
  };
}
