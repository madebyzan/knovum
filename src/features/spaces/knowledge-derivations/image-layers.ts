import { Channel } from "@tauri-apps/api/core";

import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { createSpaceNode } from "../canvas/node";
import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { useSpaceStore } from "../session/space-store";
import { applyDerivationOutput } from "./apply-output";
import { derivedNodePosition } from "./placement";
import { runDerivation } from "./run-derivation";

type SpaceStore = ReturnType<typeof useSpaceStore>;
export type LayerCapability =
  | { state: "ready" }
  | { state: "blocked"; reason: "missing_credential" | "secret_database_unavailable" };
interface LayerOutput {
  nodeId: string;
  data: SpaceImagePayload;
  relation: SpaceRelation;
}

export function loadImageLayerCapability() {
  return invoke<LayerCapability>("plugin:spaces|get_image_layer_capability");
}

export async function createImageLayers(
  store: SpaceStore,
  source: SpaceCanvasNode<SpaceImagePayload>,
  count: number,
  prompt?: string,
) {
  const spaceId = store.activeSpaceId;
  const sourceRevision = source.data.sourceRevision;
  if (!spaceId || !sourceRevision || !Number.isInteger(count) || count < 2 || count > 8)
    throw new Error("Image layer input is invalid");
  const base = derivedNodePosition([source], useCanvasEditorSession().selection.value.nodePlacements);
  const width = spaceNodeDimensions(source.data).width;
  const nodes = await Promise.all(
    Array.from({ length: count }, async (_, index) => {
      const assetPath = await invoke<string>("plugin:spaces|allocate_visualization_output_path");
      const title = `${source.data.title}-layer-${index + 1}`;
      return createSpaceNode(
        { kind: "image", title, fileName: `${title}.png`, assetPath, processingState: "processing" },
        { x: base.x + index * (width + 24), y: base.y },
      );
    }),
  );
  const pending = await store.addContents(nodes);
  const first = pending[0];
  if (!first) throw new Error("Image layer outputs were not created");
  const id = crypto.randomUUID();
  return runDerivation<LayerOutput[]>({
    store,
    pendingNode: first,
    sourceNodeIds: [source.id],
    task: {
      id,
      title: i18n.global.t("spaces.media.imageEditor.layers.task"),
      detail: source.data.title,
      cancel: () => invoke<void>("plugin:spaces|cancel_image_layers", { id }),
    },
    failureMessageKey: "spaces.media.imageEditor.layers.failed",
    execute: async ({ taskId }) => {
      const onStage = new Channel<string>(
        taskId
          ? (stage) =>
              useTaskStore().detail(taskId, i18n.global.t(`spaces.media.imageEditor.layers.stage.${stage}`))
          : undefined,
      );
      return invoke<LayerOutput[]>("plugin:spaces|create_image_layers", {
        request: {
          id,
          spaceId,
          sourceNodeId: source.id,
          sourceRevision,
          sourceEdit: source.data.imageEdit ?? null,
          outputNodeIds: pending.map(({ id: nodeId }) => nodeId),
          prompt: prompt?.trim() || null,
        },
        onStage,
      });
    },
    onSuccess: async (outputs, currentSpaceId) => {
      for (const output of outputs)
        applyDerivationOutput(store, { ...output, spaceId: currentSpaceId, relations: [output.relation] });
      await store.refreshActiveSpace();
    },
    onError: async () => {
      for (const node of pending)
        if (node.data.kind === "image")
          await store.saveNodeDataForSpace(spaceId, node.id, node.data, {
            ...node.data,
            processingState: "failed",
          });
      await store.refreshActiveSpace();
    },
  });
}
