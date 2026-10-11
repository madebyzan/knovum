import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { ImageExecutionTarget } from "../contracts/generated/ImageExecutionTarget";
import type { ImageSelection } from "../contracts/generated/ImageSelection";
import type { ImageSelectionPreview } from "../contracts/generated/ImageSelectionPreview";
import type { ImageSelectionPreviewRequest } from "../contracts/generated/ImageSelectionPreviewRequest";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export interface SelectionPreviewInput {
  node: SpaceCanvasNode;
  spaceId: string;
  execution: ImageExecutionTarget;
  selection: ImageSelection;
}
export async function previewImageSelection({
  node: source,
  spaceId,
  execution,
  selection,
  jobId,
}: SelectionPreviewInput & { jobId: string }) {
  if (source.data.kind !== "image" || !source.data.sourceRevision || execution.type !== "local")
    throw new Error("Local selection source is unavailable");
  const request: ImageSelectionPreviewRequest = {
    id: jobId,
    spaceId,
    nodeId: source.id,
    sourceRevision: source.data.sourceRevision,
    sourceEdit: source.data.imageEdit ?? null,
    model: execution.model,
    specId: execution.specId,
    selection,
  };
  const preview = await invokeCommand<ImageSelectionPreview>("plugin:spaces|preview_image_selection", {
    request,
  });
  return `data:image/png;base64,${preview.maskPngBase64}`;
}
