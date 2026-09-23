import { invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";

import { i18n } from "../../../foundation/i18n/i18n";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { yieldToCanvas } from "./canvas-yield";
import { baseNameFromNode } from "./node-naming";
import { beginPendingDerivation, endPendingDerivation } from "./pending-session";
import { requiredNodePosition } from "./placement";

type SpaceStore = ReturnType<typeof useSpaceStore>;

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
  const pending = await createPendingNode(store, source);
  if (!pending) return;
  beginPendingDerivation(pending.id, [source.id]);
  const derivationId = crypto.randomUUID();
  const tasks = useTaskStore();
  const taskId = tasks.start(t("spaces.creation.cutoutTask"), source.data.title);
  let started = false;
  try {
    await yieldToCanvas();
    const sourceEdit = source.data.imageEdit ?? null;
    await startDerivation(derivationId, spaceId, source, pending.id, sourceEdit);
    started = true;
    tasks.detail(taskId, t("spaces.creation.segmenting"));
    const completed = await invoke<CompletedCutout>("plugin:spaces|submit_image_cutout_derivation", {
      request: { id: derivationId, sourceEdit },
    });
    store.applyNodeDataForSpace(spaceId, completed.nodeId, completed.data);
    await store.refreshActiveSpace();
    tasks.succeed(taskId);
    store.requestReveal([completed.nodeId], "ensure-visible");
  } catch (error) {
    if (started) await failDerivation(derivationId, error);
    else if (pending.data.kind === "image") {
      await store.saveNodeDataForSpace(spaceId, pending.id, pending.data, {
        ...pending.data,
        processingState: "failed",
      });
    }
    await store.refreshActiveSpace();
    tasks.fail(taskId, String(error));
    toast.error(`${t("spaces.creation.cutoutFailed")}: ${String(error)}`);
  } finally {
    endPendingDerivation(pending.id);
  }
}

async function createPendingNode(store: SpaceStore, source: SpaceCanvasNode) {
  if (source.data.kind !== "image") return;
  const dimensions = spaceNodeDimensions(source.data);
  const base = baseNameFromNode(source);
  const data: SpaceImagePayload = {
    kind: "image",
    title: `${base}-cutout.png`,
    fileName: `${base}-cutout.png`,
    assetPath: source.data.assetPath,
    processingState: "importing",
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

function startDerivation(
  id: string,
  spaceId: string,
  source: SpaceCanvasNode,
  outputNodeId: string,
  sourceEdit: SpaceImagePayload["imageEdit"] | null,
) {
  if (source.data.kind !== "image" || !source.data.sourceRevision)
    throw new Error("Image revision is missing");
  return invoke("plugin:spaces|start_image_cutout_derivation", {
    request: {
      id,
      spaceId,
      sourceNodeId: source.id,
      outputNodeId,
      sourceRevision: source.data.sourceRevision,
      sourceEdit,
    },
  });
}

async function failDerivation(id: string, error: unknown) {
  await invoke("plugin:spaces|fail_image_cutout_derivation", { request: { id, error: String(error) } });
}
