import type { SpaceNodePayload } from "../../../model/nodes/node-payload";

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
    case "section":
      return "";
  }
}
