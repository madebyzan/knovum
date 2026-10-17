import { i18n } from "../../../foundation/i18n/instance";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageOperation } from "../contracts/generated/ImageOperation";
import type { ImageOperationCapabilities } from "../contracts/generated/ImageOperationCapabilities";
import type { ImageOperationTargetOption } from "../contracts/generated/ImageOperationTargetOption";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore } from "../session/space-store";
import { startDerivationAction } from "./start-action";

export function getImageOperationCapabilities() {
  return invokeCommand<ImageOperationCapabilities>("plugin:spaces|get_image_operation_capabilities");
}

const IMAGE_OPERATION_CONFIG = {
  upscale: {
    actionId: "upscale-image",
    capability: "imageUpscaling",
    resultKey: "spaces.media.imageEditor.results.upscale",
  },
  "multi-angle": {
    actionId: "generate-image-angle",
    capability: null,
    resultKey: "spaces.media.imageEditor.results.multiAngle",
  },
  "remove-object": {
    actionId: "remove-image-object",
    capability: "imageInpainting",
    resultKey: "spaces.media.imageEditor.results.removeObject",
  },
  "remove-object-prompt": {
    actionId: "remove-image-object",
    capability: null,
    resultKey: "spaces.media.imageEditor.results.removeObject",
  },
  "remove-selected-object": {
    actionId: "remove-image-object",
    capability: "imageInpainting",
    resultKey: "spaces.media.imageEditor.results.removeObject",
  },
  "remove-background": {
    actionId: "remove-image-background",
    capability: "imageSegmentation",
    resultKey: "spaces.media.imageEditor.results.removeBackground",
  },
  "extract-selection": {
    actionId: "extract-image-selection",
    capability: "imageSelection",
    resultKey: "spaces.media.imageEditor.results.extractSelection",
  },
  "generate-depth": {
    actionId: "generate-image-depth",
    capability: "imageDepth",
    resultKey: "spaces.media.imageEditor.results.generateDepth",
  },
} as const;

async function defaultImageTarget(operation: ImageOperation): Promise<ImageExecutionTarget> {
  const capabilities = await getImageOperationCapabilities();
  const capability = IMAGE_OPERATION_CONFIG[operation.kind].capability;
  if (capability) {
    return selectedImageTarget(capabilities.targets.filter((target) => target.capability === capability));
  }
  if (!capabilities.remote) throw new Error("Image execution target is not configured");
  return capabilities.remote;
}

export function selectedImageTarget(targets: readonly ImageOperationTargetOption[]): ImageExecutionTarget {
  const selected = targets.find((target) => target.selected);
  if (!selected || selected.availability.state !== "ready")
    throw new Error(i18n.global.t("spaces.media.imageEditor.selection.modelUnavailable"));
  return selected.execution;
}

export async function deriveImage(
  source: SpaceCanvasNode,
  operation: ImageOperation,
  options: { onOutputCreated?: () => void; execution?: ImageExecutionTarget | undefined } = {},
) {
  const store = useSpaceStore();
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  if (source.data.kind !== "image" || !source.data.knowledgeRevision)
    throw new Error("Image source revision is unavailable");
  const target = options.execution ?? (await defaultImageTarget(operation));
  if (store.activeSpaceId !== spaceId) throw new Error("Image derivation target Space changed");
  const title =
    `${source.data.title} · ${i18n.global.t(IMAGE_OPERATION_CONFIG[operation.kind].resultKey)}`.slice(0, 512);
  const run = await startDerivationAction({
    actionId: IMAGE_OPERATION_CONFIG[operation.kind].actionId,
    sources: [
      {
        nodeId: source.id,
        title: source.data.title,
        sourceRevision: source.data.knowledgeRevision,
        revisionBasis: "knowledge",
      },
    ],
    parameters: {
      operation,
      execution: target,
      title,
    },
    title,
  });
  if (run) options.onOutputCreated?.();
  return run;
}
