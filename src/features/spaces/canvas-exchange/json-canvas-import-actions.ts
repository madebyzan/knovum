import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";
import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportRequest,
} from "./json-canvas-import-contract";
import { importJsonCanvas as persistJsonCanvas } from "./json-canvas-import-repository";
import { mergeImportedJsonCanvasDocument } from "./json-canvas-mapping";
import { resolveImportedRelations } from "./json-canvas-relations";
import type { SpacePlacement } from "../canvas/navigation-requests";
import { setImportedCanvasNodesDeleted } from "../canvas/persistence/canvas-selection-repository";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import type { SpaceState } from "../session/session-state";
import {
  completeJsonCanvasPreparation,
  discardJsonCanvasPreparation,
  prepareJsonCanvasNodes,
} from "./json-canvas-node-preparation";
import type { createSpaceMutations } from "../session/actions/node-mutations";

interface JsonCanvasImportContext {
  ensureActiveSpacePersisted(): Promise<void>;
  mutations: ReturnType<typeof createSpaceMutations>;
  placement: SpacePlacement;
  state: SpaceState;
}

export function createJsonCanvasImportActions(context: JsonCanvasImportContext) {
  const assetProcessing = { applyNodeDataForSpace: context.mutations.applyNodeDataForSpace };
  async function importJsonCanvas(request: CanvasJsonImportRequest): Promise<CanvasJsonImportCommitResult> {
    if (!request.plan.nodes.length) throw new Error("JSON Canvas document contains no nodes to import");
    const spaceId = context.state.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active to import JSON Canvas content");
    const preparation = await prepareJsonCanvasNodes(request);
    const nodes = preparation.nodes;
    const committed = await (async () => {
      try {
        if (context.state.activeSpaceId.value !== spaceId) {
          throw new Error("JSON Canvas import target changed");
        }
        await context.ensureActiveSpacePersisted();
        if (context.state.activeSpaceId.value !== spaceId) {
          throw new Error("JSON Canvas import target changed");
        }
        for (const item of nodes) {
          item.node.data = prepareSpaceKnowledgePayload(validateSpaceNodePayload(item.node.data));
        }
        const prepared = nodes;
        const importedNodes = prepared.map(({ sourceId, node }) => ({ sourceId, nodeId: node.id }));
        const document = mergeImportedJsonCanvasDocument(request.document, {
          plan: request.plan,
          importedNodes,
          backgroundPaths: preparation.backgroundPaths,
        });
        const importedRelations = resolveImportedRelations(request.plan.edges, importedNodes, spaceId);
        const persisted = await persistJsonCanvas({
          baseRevision: request.baseRevision,
          document,
          nodes: prepared,
          relations: importedRelations,
          spaceId,
        });
        return { ...persisted, document, importedRelations, prepared };
      } catch (error) {
        return discardJsonCanvasPreparation(preparation, error);
      }
    })();
    if (context.state.activeSpaceId.value !== spaceId) {
      completeJsonCanvasPreparation(preparation);
      throw new Error("JSON Canvas import target changed");
    }
    const imported = committed.prepared.map(({ node }) => node);
    context.state.nodes.value = [...context.state.nodes.value, ...imported];
    context.state.relations.value = [...context.state.relations.value, ...committed.importedRelations];
    context.mutations.commitPendingItems(spaceId, imported, "none");
    context.placement.requestReveal(imported.map(({ id }) => id));
    for (const node of imported) {
      if (isSpaceManagedAssetPayload(node.data)) {
        scheduleSpaceAssetProcessing(assetProcessing, spaceId, node.id, node.data);
      }
    }
    completeJsonCanvasPreparation(preparation);
    const importedIds = imported.map(({ id }) => id);
    const importedRelationIds = committed.importedRelations.map(({ id }) => id);
    return {
      importedNodes: committed.importedNodes,
      importedRelations: committed.importedRelations,
      document: committed.document,
      revision: committed.revision,
      history: {
        undo: async (input) => {
          const result = await setImportedCanvasNodesDeleted({
            ...input,
            spaceId,
            nodeIds: importedIds,
            relationIds: importedRelationIds,
            deleted: true,
          });
          if (context.state.activeSpaceId.value === spaceId) {
            const removed = new Set(importedIds);
            context.state.nodes.value = context.state.nodes.value.filter(({ id }) => !removed.has(id));
            context.state.relations.value = context.state.relations.value.filter(
              ({ id }) => !importedRelationIds.includes(id),
            );
          }
          return result;
        },
        redo: async (input) => {
          const result = await setImportedCanvasNodesDeleted({
            ...input,
            spaceId,
            nodeIds: importedIds,
            relationIds: importedRelationIds,
            deleted: false,
          });
          if (context.state.activeSpaceId.value === spaceId) {
            context.state.nodes.value = [...context.state.nodes.value, ...imported];
            context.state.relations.value = [
              ...context.state.relations.value,
              ...committed.importedRelations,
            ];
          }
          context.mutations.commitPendingItems(spaceId, imported, "none");
          return result;
        },
      },
    };
  }

  return { importJsonCanvas };
}
