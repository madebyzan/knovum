import { describeSpaceNode } from "../../../model/nodes/node-description";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import type { CanvasNodePresentationContext } from "../../preview/policy";
import { projectPreviewItems } from "./canvas-preview-budget";

export function spaceNodePreviewText(
  data: SpaceNodePayload,
  context: CanvasNodePresentationContext,
): string | undefined {
  switch (data.kind) {
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
