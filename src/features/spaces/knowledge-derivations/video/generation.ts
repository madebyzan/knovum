import { Channel } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";

import { loadAiExecutionTarget, aiExecutionTargetPayload } from "../../../../foundation/ai/ai-execution";
import type { ExecutionTarget } from "../../../../foundation/ai/schema/models";
import { i18n } from "../../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload, SpaceVideoPayload } from "../../model/nodes/node-payload";
import type { SpaceRelation } from "../../model/relations/relation";
import type { useSpaceStore } from "../../session/space-store";
import { applyDerivationOutput } from "../apply-output";
import { yieldToCanvas } from "../canvas-yield";
import { beginPendingDerivation, endPendingDerivation } from "../pending-session";
import { derivedNodePosition } from "../placement";

export interface VideoDraft {
  title: string;
  prompt: string;
  aspectRatio: "16:9" | "9:16" | "1:1";
  durationSeconds: 5 | 10 | 15;
  textTarget?: ExecutionTarget | undefined;
  target?: ExecutionTarget | undefined;
}
interface Output {
  nodeId: string;
  data: SpaceVideoPayload;
  relations: SpaceRelation[];
}
type Store = ReturnType<typeof useSpaceStore>;
export interface VideoInputs {
  document: SpaceCanvasNode & { data: { kind: "markdown"; title: string; knowledgeRevision: string } };
  images: Array<SpaceCanvasNode & { data: SpaceImagePayload & { sourceRevision: string } }>;
}

interface GenerateVideoOptions {
  store: Store;
  inputs: VideoInputs;
  draft: VideoDraft;
}

export function resolveVideoInputs(nodes: readonly SpaceCanvasNode[]): VideoInputs | undefined {
  if (nodes.length === 0 || nodes.length > 11) return;
  const document = nodes.find((node) => node.data.kind === "markdown");
  const images = nodes.filter((node) => node.data.kind === "image");
  if (nodes.length !== images.length + 1 || document?.data.kind !== "markdown") return;
  if (!document.data.knowledgeRevision) return;
  if (images.some((node) => node.data.kind !== "image" || !node.data.sourceRevision)) return;
  return { document, images } as VideoInputs;
}

export async function generateVideoFromDocument({ store, inputs, draft }: GenerateVideoOptions) {
  const source = inputs.document;
  if (!store.activeSpaceId) return;
  const spaceId = store.activeSpaceId;
  const assetPath = await invokeCommand<string>("plugin:spaces|allocate_video_output_path");
  const [output] = await store.addContent(
    {
      kind: "video",
      title: draft.title,
      fileName: `${draft.title}.mp4`,
      assetPath,
      processingState: "processing",
    },
    derivedNodePosition([source, ...inputs.images], useCanvasEditorSession().selection.value.nodePlacements),
    { reveal: true },
  );
  if (!output) throw new Error("Video output node was not created");
  beginPendingDerivation(output.id, [source.id, ...inputs.images.map(({ id }) => id)]);
  const tasks = useTaskStore();
  const derivationId = crypto.randomUUID();
  const taskId = tasks.start(i18n.global.t("spaces.creation.video.generation.task"), draft.title, () =>
    invokeCommand("plugin:spaces|cancel_video_generation", { derivationId }),
  );
  const stages = new Channel<string>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes its events through onmessage.
  stages.onmessage = (stage) => tasks.detail(taskId, i18n.global.t(`spaces.creation.stages.${stage}`));
  try {
    await yieldToCanvas();
    const target = await loadAiExecutionTarget("video-generation", draft.target);
    const completed = await invokeCommand<Output>("plugin:spaces|generate_video", {
      request: {
        id: derivationId,
        spaceId,
        outputNodeId: output.id,
        inputs: [
          { nodeId: source.id, role: "primary", sourceRevision: source.data.knowledgeRevision },
          ...inputs.images.map((image) => ({
            nodeId: image.id,
            role: "source",
            sourceRevision: image.data.sourceRevision,
          })),
        ],
        prompt: draft.prompt,
        aspectRatio: draft.aspectRatio,
        durationSeconds: draft.durationSeconds,
        target: aiExecutionTargetPayload(target),
      },
      onStage: stages,
    });
    applyDerivationOutput(store, { ...completed, spaceId });
    tasks.succeed(taskId);
    return store.nodes.find((node) => node.id === completed.nodeId);
  } catch (error) {
    await store.refreshActiveSpace();
    tasks.fail(taskId, String(error));
    toast.error(`${i18n.global.t("spaces.creation.video.generation.failed")}: ${String(error)}`);
    return undefined;
  } finally {
    endPendingDerivation(output.id);
  }
}
