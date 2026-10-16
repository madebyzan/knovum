import { Channel } from "@tauri-apps/api/core";

import { i18n } from "../../../foundation/i18n/instance";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { getCanvasEditorSession } from "../canvas/editor-session";
import { createSpaceNode } from "../canvas/node";
import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import { useSpaceStore } from "../session/space-store";
import { applyDerivationOutput } from "./apply-output";
import { derivedNodePosition } from "./placement";
import { runDerivation } from "./run-derivation";

interface LayerOutput {
  nodeId: string;
  data: SpaceImagePayload;
  relation: SpaceRelation;
}
interface LayerPlan {
  planId: string;
  layerCount: number;
}
interface SplitSource {
  spaceId: string;
  sourceNodeId: string;
  sourceRevision: string;
  sourceEdit: unknown;
}

/**
 * Splits an image into layer nodes. The layer count is a property of the image rather than an
 * input, so discovery runs first and the output nodes are created once it reports how many layers
 * the image carries. Both phases share one task, so this stays a single user action.
 */
export async function splitImageLayers(source: SpaceCanvasNode<SpaceImagePayload>) {
  const store = useSpaceStore();
  const tasks = useTaskStore();
  const spaceId = store.activeSpaceId;
  const sourceRevision = source.data.sourceRevision;
  if (!spaceId || !sourceRevision) throw new Error("Image layer input is invalid");
  const splitId = crypto.randomUUID();
  const split: SplitSource = {
    spaceId,
    sourceNodeId: source.id,
    sourceRevision,
    sourceEdit: source.data.imageEdit ?? null,
  };
  let pending: SpaceCanvasNode[] = [];
  return runDerivation<LayerOutput[]>({
    spaceId,
    task: {
      id: splitId,
      title: i18n.global.t("spaces.media.imageEditor.layers.task"),
      statusMessage: source.data.title,
      cancel: () => invokeCommand<void>("plugin:spaces|cancel_image_layer_generation", { splitId }),
    },
    failureMessageKey: "spaces.media.imageEditor.layers.failed",
    execute: async ({ taskId }) => {
      const { layerCount } = await invokeCommand<LayerPlan>("plugin:spaces|plan_image_layers", {
        request: { planId: splitId, ...split },
        onStage: stageChannel(tasks, taskId),
      });
      pending = await createLayerNodes(source, layerCount, spaceId);
      return invokeCommand<LayerOutput[]>("plugin:spaces|generate_image_layers", {
        request: {
          generationId: splitId,
          planId: splitId,
          ...split,
          outputNodeIds: pending.map(({ id: nodeId }) => nodeId),
        },
      });
    },
    onSuccess: async (outputs, currentSpaceId) => {
      for (const output of outputs)
        applyDerivationOutput({ ...output, spaceId: currentSpaceId, relations: [output.relation] });
      if (store.activeSpaceId === currentSpaceId) await store.refreshActiveSpace();
    },
    onError: async () => {
      for (const node of pending)
        if (node.data.kind === "image")
          await store.saveNodeContentForSpace(spaceId, {
            id: node.id,
            expectedContent: node.data,
            data: { ...node.data, processingState: "failed" },
          });
      if (store.activeSpaceId === spaceId) await store.refreshActiveSpace();
    },
  });
}

function stageChannel(tasks: ReturnType<typeof useTaskStore>, taskId: string | undefined) {
  return new Channel<string>(
    taskId
      ? (stage) =>
          tasks.setStatusMessage(taskId, i18n.global.t(`spaces.media.imageEditor.layers.stage.${stage}`))
      : undefined,
  );
}

async function createLayerNodes(
  source: SpaceCanvasNode<SpaceImagePayload>,
  layerCount: number,
  spaceId: string,
) {
  const store = useSpaceStore();
  const base = derivedNodePosition([source], getCanvasEditorSession().selection.value.nodePlacements);
  const width = spaceNodeDimensions(source.data).width;
  const nodes = await Promise.all(
    Array.from({ length: layerCount }, async (_, index) => {
      const assetPath = await invokeCommand<string>("plugin:spaces|allocate_visualization_output_path");
      const title = `${source.data.title}-layer-${index + 1}`;
      return createSpaceNode(
        { kind: "image", title, fileName: `${title}.png`, assetPath, processingState: "processing" },
        { x: base.x + index * (width + 24), y: base.y },
      );
    }),
  );
  if (store.activeSpaceId !== spaceId) throw new Error("Image layer target Space changed");
  const pending = await store.addContents(nodes);
  if (pending.length !== layerCount) throw new Error("Image layer outputs were not created");
  return pending;
}
