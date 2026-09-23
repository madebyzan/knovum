import { Channel, invoke } from "@tauri-apps/api/core";

import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { useSpaceStore } from "../session/space-store";
import { applyDerivationOutput } from "./apply-output";
import { derivedNodePosition } from "./placement";
import { runDerivation } from "./run-derivation";

type SpaceStore = ReturnType<typeof useSpaceStore>;
export type ImageOperation =
  | { kind: "upscale"; targetLongEdge: 2048 | 4096 | 8192 }
  | {
      kind: "multi-angle";
      mode: "subject" | "camera";
      yaw: number;
      pitch: number;
      zoom: "close" | "medium" | "wide";
    };

export type ImageOperationCapability =
  | { state: "ready" }
  | { state: "blocked"; reason: "configuration_incomplete" | "provider_unsupported" | "credential_unavailable" };

interface ImageOperationCapabilities {
  upscale: ImageOperationCapability;
  multiAngle: ImageOperationCapability;
}

interface ImageOperationOutput {
  nodeId: string;
  data: SpaceImagePayload;
  relation: SpaceRelation;
}

export function loadImageOperationCapabilities() {
  return invoke<ImageOperationCapabilities>("plugin:spaces|get_image_operation_capabilities");
}

export async function createImageOperation(
  store: SpaceStore,
  source: SpaceCanvasNode<SpaceImagePayload>,
  operation: ImageOperation,
) {
  const spaceId = store.activeSpaceId;
  const sourceRevision = source.data.sourceRevision;
  if (!spaceId || !sourceRevision) throw new Error("Image source revision is unavailable");
  const outputPath = await invoke<string>("plugin:spaces|allocate_visualization_output_path");
  const label =
    operation.kind === "upscale"
      ? `${operation.targetLongEdge / 1024}K`
      : `${operation.mode}-${operation.yaw}-${operation.pitch}-${operation.zoom}`;
  const title = `${source.data.title}-${label}`;
  const [pending] = await store.addContent(
    {
      kind: "image",
      title,
      fileName: `${title}.png`,
      assetPath: outputPath,
      processingState: "processing",
    },
    derivedNodePosition([source], useCanvasEditorSession().selection.value.nodePlacements),
    { reveal: true },
  );
  if (!pending) throw new Error("Image operation output node was not created");
  const id = crypto.randomUUID();
  return runDerivation<ImageOperationOutput>({
    store,
    pendingNode: pending,
    sourceNodeIds: [source.id],
    task: {
      id,
      title: i18n.global.t("spaces.media.imageOperationTask"),
      detail: title,
      cancel: () => cancelAiJob(id),
    },
    failureMessageKey: "spaces.media.imageOperationFailed",
    execute: async ({ taskId }) => {
      const stages = new Channel<string>();
      if (taskId) {
        const tasks = useTaskStore();
        stages.onmessage = (stage) => tasks.detail(taskId, i18n.global.t(`spaces.media.operationStage.${stage}`));
      }
      return invoke<ImageOperationOutput>("plugin:spaces|create_image_operation", {
        request: {
          id,
          spaceId,
          sourceNodeId: source.id,
          outputNodeId: pending.id,
          sourceRevision,
          sourceEdit: source.data.imageEdit ?? null,
          operation,
        },
        onStage: stages,
      });
    },
    onSuccess: (completed, currentSpaceId) => {
      applyDerivationOutput(store, {
        ...completed,
        spaceId: currentSpaceId,
        relations: [completed.relation],
      });
    },
    onError: async () => {
      if (pending.data.kind === "image")
        await store.saveNodeDataForSpace(spaceId, pending.id, {
          ...pending.data,
          processingState: "failed",
        });
      await store.refreshActiveSpace();
    },
  });
}
