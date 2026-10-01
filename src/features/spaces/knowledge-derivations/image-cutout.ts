import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { yieldToCanvas } from "./canvas-yield";
import { baseNameFromNode } from "./node-naming";
import { requiredNodePosition } from "./placement";

type SpaceStore = Pick<
  ReturnType<typeof useSpaceStore>,
  | "activeSpaceId"
  | "nodes"
  | "addContent"
  | "applyNodeContentForSpace"
  | "refreshActiveSpace"
  | "requestReveal"
  | "saveNodeContentForSpace"
>;

interface CompletedCutout {
  nodeId: string;
  data: SpaceImagePayload;
}

export function canRemoveImageBackground(node: SpaceCanvasNode | undefined): boolean {
  return node?.data.kind === "image" && Boolean(node.data.assetPath && node.data.sourceRevision);
}

export async function createImageCutout(store: SpaceStore, source: SpaceCanvasNode) {
  const t = i18n.global.t;
  if (!canRemoveImageBackground(source) || source.data.kind !== "image") return;
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const sourceEdit = source.data.imageEdit ?? null;
  if (!(await checkCutoutSource(spaceId, source, sourceEdit))) return;
  const pending = await createPendingNode(store, source);
  if (!pending) return;
  const derivationId = crypto.randomUUID();
  const tasks = useTaskStore();
  const taskId = tasks.start(t("spaces.creation.image.cutout.task"), source.data.title);
  try {
    await yieldToCanvas();
    tasks.setStatusMessage(taskId, t("spaces.creation.image.cutout.segmenting"));
    const completed = await invokeCommand<CompletedCutout>("plugin:spaces|run_image_cutout_derivation", {
      request: {
        id: derivationId,
        spaceId,
        sourceNodeId: source.id,
        outputNodeId: pending.id,
        sourceRevision: source.data.sourceRevision,
        sourceEdit,
      },
    });
    store.applyNodeContentForSpace(spaceId, completed.nodeId, completed.data);
    await store.refreshActiveSpace();
    tasks.succeed(taskId);
    store.requestReveal([completed.nodeId], "ensure-visible");
  } catch (error) {
    try {
      await markPendingCutoutFailed(store, spaceId, pending.id);
    } finally {
      tasks.fail(taskId, getErrorMessage(error));
      toast.error(`${t("spaces.creation.image.cutout.failed")}: ${getErrorMessage(error)}`);
    }
  }
}

async function markPendingCutoutFailed(store: SpaceStore, spaceId: string, outputNodeId: string) {
  await store.refreshActiveSpace();
  if (store.activeSpaceId !== spaceId) return;
  const current = store.nodes.find(({ id }) => id === outputNodeId);
  if (current?.data.kind === "image" && current.data.processingState === "processing") {
    await store.saveNodeContentForSpace(spaceId, current.id, current.data, {
      ...current.data,
      processingState: "failed",
    });
  }
}

async function checkCutoutSource(
  spaceId: string,
  source: SpaceCanvasNode,
  sourceEdit: SpaceImagePayload["imageEdit"] | null,
): Promise<boolean> {
  try {
    const available = await invokeCommand<boolean>("plugin:spaces|check_image_cutout_source", {
      request: {
        spaceId,
        sourceNodeId: source.id,
        sourceRevision: source.data.sourceRevision,
        sourceEdit,
      },
    });
    if (available) return true;
    toast.error(i18n.global.t("spaces.creation.image.cutout.sourceMissing"));
  } catch (error) {
    toast.error(`${i18n.global.t("spaces.creation.image.cutout.failed")}: ${getErrorMessage(error)}`);
  }
  return false;
}

async function createPendingNode(store: SpaceStore, source: SpaceCanvasNode) {
  if (source.data.kind !== "image") return;
  const dimensions = spaceNodeDimensions(source.data);
  const base = baseNameFromNode(source);
  const assetPath = await invokeCommand<string>("plugin:spaces|allocate_visualization_output_path");
  const data: SpaceImagePayload = {
    kind: "image",
    title: `${base}-cutout.png`,
    fileName: `${base}-cutout.png`,
    assetPath,
    processingState: "processing",
  };
  const sourcePosition = requiredNodePosition(
    useCanvasEditorSession().selection.value.nodePlacements,
    source.id,
  );
  const [node] = await store.addContent(
    data,
    { x: sourcePosition.x + dimensions.width + 48, y: sourcePosition.y },
    { reveal: true },
  );
  return node;
}
