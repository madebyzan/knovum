import type { SpaceEditorDocument } from "../contract/canvas-document";

export function contentAddedBetween(before: SpaceEditorDocument, after: SpaceEditorDocument) {
  const nodeIds = new Set(before.nodes.map(({ nodeId }) => nodeId));
  return {
    nodes: after.nodes.filter(({ nodeId }) => !nodeIds.has(nodeId)),
  };
}

export function removeCanvasContent(
  document: SpaceEditorDocument,
  nodeIds: ReadonlySet<string>,
): SpaceEditorDocument {
  return {
    ...document,
    nodes: document.nodes.filter(({ nodeId }) => !nodeIds.has(nodeId)),
  };
}

export function removeCanvasSelection(
  document: SpaceEditorDocument,
  nodeIds: ReadonlySet<string>,
  relationIds: ReadonlySet<string>,
): SpaceEditorDocument {
  const content = removeCanvasContent(document, nodeIds);
  const relationStyles = content.relationStyles?.filter(({ relationId }) => !relationIds.has(relationId));
  const { relationStyles: _current, ...documentWithoutRelationStyles } = content;
  return relationStyles?.length
    ? { ...documentWithoutRelationStyles, relationStyles }
    : documentWithoutRelationStyles;
}

export function restoreCanvasContent(
  document: SpaceEditorDocument,
  target: SpaceEditorDocument,
  nodeIds: ReadonlySet<string>,
) {
  const nodes = restoreAtTargetIndices(document.nodes, target.nodes, nodeIds, ({ nodeId }) => nodeId);
  return { ...document, nodes };
}

function restoreAtTargetIndices<T>(
  current: readonly T[],
  target: readonly T[],
  ids: ReadonlySet<string>,
  identity: (value: T) => string,
) {
  const restored = [...current];
  for (const [index, value] of target.entries()) {
    if (ids.has(identity(value))) restored.splice(Math.min(index, restored.length), 0, value);
  }
  return restored;
}
