import type { CanvasNodePresentationContext } from "../../../canvas/contract/canvas-detail-profile";
import type { SpaceNodePayload } from "../../../model/nodes/node-payload";
import { projectPreviewItems } from "./canvas-preview-budget";

// Exhaustive domain presentation has many flat cases but no branching interaction.
// oxlint-disable-next-line eslint/complexity
export function spaceNodeSummary(data: SpaceNodePayload) {
  switch (data.kind) {
    case "note":
    case "text":
      return data.plainText;
    case "markdown":
      return data.text;
    case "website":
      return data.description ?? data.contentPreview ?? data.url;
    case "subtitle":
      return data.cues
        .slice(0, 3)
        .map(({ text }) => text)
        .join("\n");
    case "image":
    case "video":
    case "audio":
    case "file":
    case "code":
      return data.fileName;
    case "credential":
      return data.credentialKind;
    case "object":
      return data.summary ?? data.objectTypeId;
    case "portal":
      return data.targetSceneId ? "Linked scene" : "Linked space";
  }
}

// Exhaustive domain projection has many flat cases but no branching interaction.
// oxlint-disable-next-line eslint/complexity
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
    case "note":
    case "text":
      return data.plainText;
    case "markdown":
      return data.text;
    case "website":
      return data.description ?? data.contentPreview ?? data.url;
    case "image":
    case "video":
    case "audio":
    case "file":
      return data.fileName;
    case "credential":
      return data.credentialKind;
    case "portal":
      return data.targetSceneId ? "Linked scene" : "Linked space";
  }
}
