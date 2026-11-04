import type { TiptapJsonNode } from "../../../document-editing/tiptap-document";
import { CANVAS_PREVIEW_CHARACTERS_PER_ITEM, type CanvasNodePresentationContext } from "../../preview/policy";

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
    content: projectChildren(sliceBlocks(document.content, context.viewport.previewItemLimit), budget),
  };
}

function sliceBlocks(nodes: readonly TiptapJsonNode[], limit: number): readonly TiptapJsonNode[] {
  let count = 0;
  let end = 0;
  for (const [index, node] of nodes.entries()) {
    const empty = node.type === "paragraph" && (!node.content || node.content.length === 0);
    if (!empty) {
      count++;
      end = index + 1;
      if (count >= limit) break;
    }
  }
  return end > 0 ? nodes.slice(0, end) : nodes.slice(0, limit);
}

function projectChildren(
  nodes: readonly TiptapJsonNode[],
  budget: { nodes: number; characters: number },
): TiptapJsonNode[] {
  const projectedNodes: TiptapJsonNode[] = [];
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
    projectedNodes.push(projected);
  }
  return projectedNodes;
}
