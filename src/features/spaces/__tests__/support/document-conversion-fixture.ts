import { createSpaceNode } from "../../canvas/node";
import type { SpaceFilePayload } from "../../model/nodes/node-payload";

export function documentFile(extension: string) {
  return createSpaceNode<SpaceFilePayload>(
    {
      kind: "file",
      title: "Report",
      fileName: `report.${extension}`,
      assetPath: `/spaces/assets/report.${extension}`,
      sourceRevision: "sha256:current",
      knowledgeRevision: "file:current",
    },
    { x: 0, y: 0 },
    "file",
  );
}
