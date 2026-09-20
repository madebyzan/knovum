import type {
  CanvasCutSelectionRequest,
  CanvasDeleteSelectionRequest,
  CanvasNodePlacement,
} from "./contract/canvas-editor";
import {
  deleteCanvasSelection,
  restoreCanvasSelection,
} from "./persistence/canvas-selection-repository";
import { copySpaceSelectionToClipboard, duplicateSpaceSelection } from "./selection-actions";
import { writeSpaceClipboardDirect } from "../clipboard/transfer";
import { scheduleKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import type { PositionedSpaceNode } from "../model/nodes/canvas-node";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { omit } from "../organization/selection-mutations";
import type { useSpaceStore } from "../session/space-store";

type SpaceStore = ReturnType<typeof useSpaceStore>;

export function createCanvasSelectionActions(store: SpaceStore) {
  function nodesAtPlacements(placements: readonly CanvasNodePlacement[]): PositionedSpaceNode[] {
    const positions = new Map(placements.map(({ id, x, y }) => [id, { x, y }]));
    return store.nodes.flatMap((node) => {
      const position = positions.get(node.id);
      return position ? [{ node, position }] : [];
    });
  }

  async function deleteSelection(request: CanvasDeleteSelectionRequest) {
    const spaceId = requireSpaceId(store.activeSpaceId);
    const selected = new Set(request.nodeIds);
    const removedNodes = store.nodes
      .map((node, index) => ({ node, index }))
      .filter(({ node }) => selected.has(node.id));
    const deletedSections = new Set(
      removedNodes.filter(({ node }) => node.data.kind === "section").map(({ node }) => node.id),
    );
    const survivorChanges = store.nodes.flatMap((node) => {
      if (selected.has(node.id) || !node.data.sectionId || !deletedSections.has(node.data.sectionId))
        return [];
      const before = validateSpaceNodePayload(node.data);
      const after = prepareSpaceKnowledgePayload(validateSpaceNodePayload(omit(before, "sectionId")));
      return [{ id: node.id, before, after }];
    });
    const explicitlySelectedRelations = new Set(request.relationIds);
    const removedRelations = store.relations.filter(
      ({ id, sourceId, targetId }) =>
        explicitlySelectedRelations.has(id) || selected.has(sourceId) || selected.has(targetId),
    );
    const incidentRelationIds = new Set(
      removedRelations
        .filter(({ sourceId, targetId }) => selected.has(sourceId) || selected.has(targetId))
        .map(({ id }) => id),
    );
    const survivorBefore = new Map(survivorChanges.map(({ id, before }) => [id, before]));
    const survivorAfter = new Map(survivorChanges.map(({ id, after }) => [id, after]));
    const result = await deleteCanvasSelection({
      baseRevision: request.baseRevision,
      document: request.document,
      nodeIds: request.nodeIds,
      relationIds: request.relationIds.filter((id) => !incidentRelationIds.has(id)),
      spaceId,
      survivorChanges: survivorChanges.map(({ id, after: data }) => ({ id, data })),
    });
    if (store.activeSpaceId !== spaceId) throw new Error("Canvas deletion target changed");
    applyDeletedState();
    return {
      ...result,
      document: request.document,
      history: { undo: restoreDomainState, redo: deleteDomainState },
    };

    function applyDeletedState() {
      store.nodes = applyPayloads(
        store.nodes.filter(({ id }) => !selected.has(id)),
        survivorAfter,
      );
      const removedRelationIds = new Set(removedRelations.map(({ id }) => id));
      store.relations = store.relations.filter(({ id }) => !removedRelationIds.has(id));
    }

    async function restoreDomainState(input: {
      baseRevision: number;
      document: CanvasDeleteSelectionRequest["document"];
    }) {
      const restored = await restoreCanvasSelection({
        ...input,
        spaceId,
        nodeIds: request.nodeIds,
        relationIds: removedRelations.map(({ id }) => id),
        survivorChanges: survivorChanges.map(({ id, before: data }) => ({ id, data })),
      });
      const nodes = [...store.nodes];
      for (const { node, index } of removedNodes) nodes.splice(Math.min(index, nodes.length), 0, node);
      store.nodes = applyPayloads(nodes, survivorBefore);
      store.relations = [...store.relations, ...removedRelations];
      scheduleKnowledgeIndex(spaceId, request.nodeIds);
      return restored;
    }

    async function deleteDomainState(input: {
      baseRevision: number;
      document: CanvasDeleteSelectionRequest["document"];
    }) {
      const deleted = await deleteCanvasSelection({
        ...input,
        spaceId,
        nodeIds: request.nodeIds,
        relationIds: request.relationIds.filter((id) => !incidentRelationIds.has(id)),
        survivorChanges: survivorChanges.map(({ id, after: data }) => ({ id, data })),
      });
      applyDeletedState();
      return deleted;
    }
  }

  async function cutSelection(request: CanvasCutSelectionRequest) {
    const nodes = nodesAtPlacements(request.nodePlacements);
    if (!(await writeSpaceClipboardDirect(nodes))) throw new Error("Canvas cut requires selected nodes");
    return deleteSelection(request);
  }

  return {
    copyNodes: (placements: readonly CanvasNodePlacement[]) =>
      copySpaceSelectionToClipboard(nodesAtPlacements(placements)),
    cutSelection,
    duplicateNodes: (placements: readonly CanvasNodePlacement[]) =>
      duplicateSpaceSelection(store, nodesAtPlacements(placements)),
    deleteSelection,
  };
}

function applyPayloads(
  nodes: readonly PositionedSpaceNode["node"][],
  payloads: ReadonlyMap<string, PositionedSpaceNode["node"]["data"]>,
) {
  const updated = [...nodes];
  for (const [index, node] of updated.entries()) {
    const data = payloads.get(node.id);
    if (data) updated[index] = { ...node, data };
  }
  return updated;
}

function requireSpaceId(spaceId: string | undefined) {
  if (!spaceId) throw new Error("A Space must be active to delete canvas content");
  return spaceId;
}
