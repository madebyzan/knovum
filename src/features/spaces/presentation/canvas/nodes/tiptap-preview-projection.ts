import {
  CANVAS_PREVIEW_CHARACTERS_PER_ITEM,
  type CanvasNodePresentationContext,
} from "../../../canvas/preview/policy";
import type { TiptapJsonNode } from "../../../editor/tiptap-document";

export function projectTiptapPreview(
  document: TiptapJsonNode,
  context: CanvasNodePresentationContext,
): TiptapJsonNode {
  if (context.mode === "interactive" || !document.content) return document;
  const budget = {
    nodes: context.viewport.previewItemLimit * 32,
    characters: context.viewport.previewItemLimit * CANVAS_PREVIEW_CHARACTERS_PER_ITEM,
  };
  return {
    ...document,
    content: projectChildren(document.content.slice(0, context.viewport.previewItemLimit), budget),
  };
}

function projectChildren(
  nodes: readonly TiptapJsonNode[],
  budget: { nodes: number; characters: number },
): TiptapJsonNode[] {
  const result: TiptapJsonNode[] = [];
  for (const node of nodes) {
    if (budget.nodes <= 0 || budget.characters <= 0) break;
    budget.nodes--;
    const projected = { ...node };
    if (node.text !== undefined) {
      projected.text = node.text.slice(0, budget.characters);
      budget.characters -= projected.text.length;
    }
    if (node.content) {
      projected.content = projectChildren(node.content, budget);
      if (node.type !== "paragraph" && projected.content.length === 0) continue;
    }
    result.push(projected);
  }
  return result;
}
