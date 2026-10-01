import type { JsonCanvasDocumentV1 } from "../contracts/generated/JsonCanvasDocumentV1";
import type { JsonCanvasNodeV1 } from "../contracts/generated/JsonCanvasNodeV1";
import type { SpaceRelation } from "../model/relations/relation";
import type { StructureLayout } from "../model/structure/schema/structure";
import { jsonColorForCanvasFill } from "./json-canvas-colors";
import type { JsonCanvasNodeContentV1 } from "./json-canvas-import-contract";

export interface JsonCanvasExportNodeV1 {
  id: string;
  frame: { x: number; y: number; width: number; height: number };
  appearance?: Extract<StructureLayout, { kind: "content" | "section" }>["appearance"];
  fileSubpath?: string;
  content: JsonCanvasNodeContentV1 | { type: "group"; label?: string };
}

export function exportJsonCanvasDocument(
  nodes: readonly JsonCanvasExportNodeV1[],
  relations: readonly SpaceRelation[],
): JsonCanvasDocumentV1 {
  const nodeIds = new Set(nodes.filter(({ content }) => content.type !== "group").map(({ id }) => id));
  return {
    nodes: nodes.map(exportNode),
    edges: relations.flatMap((relation) =>
      relation.status === "active" && nodeIds.has(relation.sourceId) && nodeIds.has(relation.targetId)
        ? [
            {
              id: relation.id,
              fromNode: relation.sourceId,
              toNode: relation.targetId,
              toEnd: "arrow" as const,
              ...(relation.rationale ? { label: relation.rationale } : {}),
            },
          ]
        : [],
    ),
  };
}

function exportNode(source: JsonCanvasExportNodeV1): JsonCanvasNodeV1 {
  const node = { id: source.id, ...standardFrame(source.frame), ...source.content } as JsonCanvasNodeV1;
  applyExportAppearance(node, source);
  applyExportFileSubpath(node, source.fileSubpath);
  return node;
}

function applyExportAppearance(node: JsonCanvasNodeV1, source: JsonCanvasExportNodeV1) {
  const appearance = source.appearance;
  if (appearance?.fill) node.color = jsonColorForCanvasFill(appearance.fill);
  if (appearance?.customFill) node.color = appearance.customFill;
  if (node.type !== "group") return;
  if (appearance?.background) node.background = appearance.background;
  if (appearance?.backgroundStyle) node.backgroundStyle = appearance.backgroundStyle;
}

function applyExportFileSubpath(node: JsonCanvasNodeV1, subpath: string | undefined) {
  if (node.type === "file" && subpath) node.subpath = subpath;
}

function standardFrame(frame: JsonCanvasExportNodeV1["frame"]) {
  const geometry = Object.fromEntries(
    Object.entries(frame).map(([key, value]) => [key, Math.round(value)]),
  ) as JsonCanvasExportNodeV1["frame"];
  if (!Object.values(geometry).every(Number.isSafeInteger) || geometry.width <= 0 || geometry.height <= 0)
    throw new Error("Canvas frame cannot be represented by JSON Canvas 1.0 integer geometry");
  return geometry;
}
