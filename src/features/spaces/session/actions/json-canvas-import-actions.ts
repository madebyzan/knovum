import { i18n } from "../../../../foundation/i18n/i18n";
import { scheduleSpaceAssetProcessing } from "../../assets/processing-scheduler";
import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportRequest,
} from "../../canvas-exchange/json-canvas-import-contract";
import { importJsonCanvas as persistJsonCanvas } from "../../canvas-exchange/json-canvas-import-repository";
import { resolveImportedRelations } from "../../canvas-exchange/json-canvas-relations";
import { mergeImportedJsonCanvasStructure } from "../../canvas-exchange/json-canvas-structure-import";
import type { SpacePlacement } from "../../canvas/navigation-requests";
import { prepareSpaceKnowledgePayload } from "../../knowledge/model/node-knowledge";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import { validateSpaceNodePayload } from "../../model/nodes/schema/node-payload";
import type { SpaceState } from "../session-state";
import {
  completeJsonCanvasPreparation,
  discardJsonCanvasPreparation,
  prepareJsonCanvasNodes,
} from "./json-canvas-node-preparation";
import type { createSpaceMutations } from "./node-mutations";

interface JsonCanvasImportContext {
  ensureActiveSpacePersisted(): Promise<void>;
  mutations: ReturnType<typeof createSpaceMutations>;
  placement: SpacePlacement;
  state: SpaceState;
}

export function createJsonCanvasImportActions(context: JsonCanvasImportContext) {
  const assetProcessing = { applyNodeDataForSpace: context.mutations.applyNodeDataForSpace };
  async function importJsonCanvas(request: CanvasJsonImportRequest): Promise<CanvasJsonImportCommitResult> {
    if (!request.plan.contents.length && !request.plan.sections.length)
      throw new Error("JSON Canvas document contains no nodes to import");
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
        for (const preparedNode of nodes) {
          preparedNode.node.data = prepareSpaceKnowledgePayload(
            validateSpaceNodePayload(preparedNode.node.data),
          );
        }
        const prepared = nodes;
        const importedNodes = prepared.map(({ sourceId, node }) => ({ sourceId, nodeId: node.id }));
        const snapshot = mergeImportedJsonCanvasStructure(context.state.editSnapshot.value, request.plan, {
          contentIds: preparation.contentIds,
          sectionIds: preparation.sectionIds,
          backgroundPaths: preparation.backgroundPaths,
          defaultSectionTitle: i18n.global.t("spaces.canvas.untitledSection"),
        });
        const importedRelations = resolveImportedRelations(request.plan.edges, importedNodes, spaceId);
        const persisted = await persistJsonCanvas({
          baseRevision: request.baseRevision,
          mutationId: crypto.randomUUID(),
          snapshot,
          nodes: prepared,
          relations: importedRelations,
          spaceId,
        });
        return { ...persisted, snapshot, importedRelations, prepared };
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
    context.state.editSnapshot.value = {
      ...committed.snapshot,
      editRevision: committed.revision,
    };
    context.mutations.commitPendingContent(spaceId, imported);
    context.placement.requestReveal(imported.map(({ id }) => id));
    for (const node of imported) {
      if (isSpaceManagedAssetPayload(node.data)) {
        scheduleSpaceAssetProcessing(assetProcessing, spaceId, node.id, node.data);
      }
    }
    completeJsonCanvasPreparation(preparation);
    return {
      importedNodes: committed.importedNodes,
      importedRelations: committed.importedRelations,
      snapshot: context.state.editSnapshot.value,
      revision: committed.revision,
    };
  }

  return { importJsonCanvas };
}
