import { parse } from "pathe";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export function baseNameFromNode(node: SpaceCanvasNode): string {
  const fileName =
    "fileName" in node.data && typeof node.data.fileName === "string" ? node.data.fileName : "";
  return (fileName ? parse(fileName).name : "") || node.data.title;
}
