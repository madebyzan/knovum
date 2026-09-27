import type { ImageAspectRatio } from "../contracts/generated/ImageAspectRatio";
export type { ImageAspectRatio } from "../contracts/generated/ImageAspectRatio";
import { Channel } from "@tauri-apps/api/core";

import { aiExecutionTargetPayload, loadAiExecutionTarget } from "../../../foundation/ai/ai-execution";
import type { ExecutionTarget } from "../../../foundation/ai/ai-models";
import { cancelAiJob } from "../../../foundation/ai/ai-runtime";
import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand as invoke } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type { useSpaceStore } from "../session/space-store";
import { applyDerivationOutput } from "./apply-output";
import type { GroundedDocumentSource } from "./catalog/grounded-document";
import { derivedNodePosition } from "./placement";
import { runDerivation } from "./run-derivation";

type SpaceStore = ReturnType<typeof useSpaceStore>;
export interface VisualizationDraft {
  title: string;
  intent: string;
  style: string;
  aspectRatio: ImageAspectRatio;
  target?: ExecutionTarget | undefined;
}

const imageAspectRatios: ReadonlySet<unknown> = new Set(["1:1", "3:2", "2:3", "16:9", "9:16"]);

export function isImageAspectRatio(value: unknown): value is ImageAspectRatio {
  return imageAspectRatios.has(value);
}
interface VisualizationOutput {
  nodeId: string;
  data: SpaceImagePayload;
  relations: SpaceRelation[];
}
interface GenerateVisualizationContext {
  store: SpaceStore;
  selected: readonly SpaceCanvasNode[];
  sources: readonly GroundedDocumentSource[];
  draft: VisualizationDraft;
}

export async function generateVisualization(context: GenerateVisualizationContext) {
  const { store, selected, sources, draft } = context;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const assetPath = await invoke<string>("plugin:spaces|allocate_visualization_output_path");
  const pending: SpaceImagePayload = {
    kind: "image",
    title: draft.title.trim(),
    fileName: `${draft.title.trim()}.png`,
    assetPath,
    processingState: "processing",
  };
  const [output] = await store.addContent(
    pending,
    derivedNodePosition(selected, useCanvasEditorSession().selection.value.nodePlacements),
    {
      reveal: true,
    },
  );
  if (!output) throw new Error("Visualization output node was not created");
  const jobId = crypto.randomUUID();

  return runDerivation<VisualizationOutput>({
    store,
    pendingId: output.id,
    sourceNodeIds: sources.map(({ nodeId }) => nodeId),
    task: {
      id: jobId,
      title: i18n.global.t("spaces.creation.image.visualization.task"),
      detail: draft.title.trim(),
      cancel: () => cancelAiJob(jobId),
    },
    failureMessageKey: "spaces.creation.image.visualization.failed",
    execute: async ({ taskId }) => {
      const stages = new Channel<string>();
      if (taskId) {
        const tasks = useTaskStore();
        // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes its events through onmessage.
        stages.onmessage = (stage) => tasks.detail(taskId, i18n.global.t(`spaces.creation.stages.${stage}`));
      }
      const target = await loadAiExecutionTarget("image-generation", draft.target);
      return invoke<VisualizationOutput>("plugin:spaces|generate_visualization", {
        request: {
          id: jobId,
          spaceId,
          outputNodeId: output.id,
          inputs: sources.map(({ nodeId, role, sourceRevision }) => ({ nodeId, role, sourceRevision })),
          intent: draft.intent.trim(),
          style: draft.style,
          aspectRatio: draft.aspectRatio,
          target: aiExecutionTargetPayload(target),
        },
        onStage: stages,
      });
    },
    onSuccess: (completed, currentSpaceId) => {
      applyDerivationOutput(store, { ...completed, spaceId: currentSpaceId });
    },
  });
}
