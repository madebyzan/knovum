import { Channel, invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";

import { loadAiExecutionTarget, aiExecutionTargetPayload } from "../../../../foundation/ai/ai-execution";
import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import { i18n } from "../../../../foundation/i18n/i18n";
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

export interface KnowledgeVideoDraft {
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
export interface KnowledgeVideoInputs {
  document: SpaceCanvasNode & { data: { kind: "markdown"; title: string; knowledgeRevision: string } };
  images: Array<SpaceCanvasNode & { data: SpaceImagePayload & { sourceRevision: string } }>;
}

interface CreateKnowledgeVideoOptions {
  store: Store;
  inputs: KnowledgeVideoInputs;
  draft: KnowledgeVideoDraft;
}

export function knowledgeVideoInputs(nodes: readonly SpaceCanvasNode[]): KnowledgeVideoInputs | undefined {
  if (nodes.length === 0 || nodes.length > 11) return;
  const document = nodes.find((node) => node.data.kind === "markdown");
  const images = nodes.filter((node) => node.data.kind === "image");
  if (nodes.length !== images.length + 1 || document?.data.kind !== "markdown") return;
  if (!document.data.knowledgeRevision) return;
  if (images.some((node) => node.data.kind !== "image" || !node.data.sourceRevision)) return;
  return { document, images } as KnowledgeVideoInputs;
}

export async function createKnowledgeVideoFromDocument({
  store,
  inputs,
  draft,
}: CreateKnowledgeVideoOptions) {
  const source = inputs.document;
  if (!store.activeSpaceId) return;
  const spaceId = store.activeSpaceId;
  const assetPath = await invoke<string>("plugin:spaces|allocate_knowledge_video_path");
  const [output] = await store.addItem(
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
  const jobId = crypto.randomUUID();
  const taskId = tasks.start(i18n.global.t("spaces.creation.videoTask"), draft.title, () =>
    invoke("plugin:spaces|cancel_knowledge_video", { jobId }),
  );
  const stages = new Channel<string>();
  // oxlint-disable-next-line unicorn/prefer-add-event-listener -- Tauri Channel exposes its events through onmessage.
  stages.onmessage = (stage) => tasks.detail(taskId, i18n.global.t(`spaces.creation.${stage}`));
  try {
    await yieldToCanvas();
    const target = await loadAiExecutionTarget("video-generation", draft.target);
    const completed = await invoke<Output>("plugin:spaces|create_knowledge_video", {
      request: {
        id: jobId,
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
    toast.error(`${i18n.global.t("spaces.creation.videoFailed")}: ${String(error)}`);
    return undefined;
  } finally {
    endPendingDerivation(output.id);
  }
}
