import type { CanvasNodePresentationContext } from "../../../canvas/preview/detail-profile";
import { describeSpaceNode } from "../../../model/nodes/node-description";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { projectPreviewItems } from "./canvas-preview-budget";

export function spaceNodePreviewText(
  data: SpaceNodePayload,
  context: CanvasNodePresentationContext,
): string | undefined {
  switch (data.kind) {
    case "subtitle":
      return projectPreviewItems(data.cues, context)
        .map(({ text }) => text)
        .join("\n");
    case "object": {
      const fields = [
        ...(data.summary ? [data.summary] : []),
        ...Object.entries(data.properties).map(([key, value]) => `${key}: ${String(value)}`),
      ];
      return projectPreviewItems(fields, context).join("\n");
    }
    case "code":
      return projectPreviewItems((data.previewText ?? data.fileName).split(/\r?\n/u), context).join("\n");
    default:
      return describeSpaceNode(data);
  }
}
