import type { SpaceRelation } from "../model/relations/relation";
import { SECTION_MINIMUM_SIZE } from "../model/structure/section-metrics";
import type { StructureLayout } from "../model/structure/structure-contract";
import { canvasFillForJsonColor, jsonColorForCanvasFill } from "./json-canvas-colors";
import type { JsonCanvasDocumentV1, JsonCanvasNodeV1 } from "./json-canvas-contract";
import type {
  JsonCanvasImportContentV1,
  JsonCanvasImportPlanV1,
  JsonCanvasImportSectionV1,
  JsonCanvasNodeContentV1,
} from "./json-canvas-import-contract";

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

export function importJsonCanvasDocument(document: JsonCanvasDocumentV1): JsonCanvasImportPlanV1 {
  const nodes = document.nodes ?? [];
  const groups = nodes.filter(isGroup);
  for (const group of groups) {
    if (group.width < SECTION_MINIMUM_SIZE.width || group.height < SECTION_MINIMUM_SIZE.height)
      throw new Error(`JSON Canvas group is too small to become a Section: ${group.id}`);
  }
  const contents = nodes.filter(
    (node): node is Exclude<JsonCanvasNodeV1, { type: "group" }> => !isGroup(node),
  );
  const contentIds = new Set(contents.map(({ id }) => id));
  for (const edge of document.edges ?? []) {
    if (!contentIds.has(edge.fromNode) || !contentIds.has(edge.toNode))
      throw new Error("JSON Canvas edges cannot target visual groups when importing as Sections");
  }
  return {
    sections: nodes.flatMap((node, order) => (isGroup(node) ? [importSection(node, order)] : [])),
    contents: nodes.flatMap((node, order) => (isGroup(node) ? [] : [importContent(node, order, groups)])),
    edges: document.edges ?? [],
  };
}

function importSection(
  node: Extract<JsonCanvasNodeV1, { type: "group" }>,
  order: number,
): JsonCanvasImportSectionV1 {
  const appearance = nodeAppearance(node);
  return {
    sourceId: node.id,
    frame: frameOf(node),
    order,
    ...(node.label ? { title: node.label } : {}),
    ...(appearance ? { appearance } : {}),
  };
}

function importContent(
  node: Exclude<JsonCanvasNodeV1, { type: "group" }>,
  order: number,
  groups: readonly Extract<JsonCanvasNodeV1, { type: "group" }>[],
): JsonCanvasImportContentV1 {
  const appearance = nodeAppearance(node);
  const containingGroups = groups.filter((group) => contains(group, node));
  if (containingGroups.length > 1) {
    throw new Error(`JSON Canvas content has ambiguous visual group membership: ${node.id}`);
  }
  const parentSectionSourceId = containingGroups[0]?.id;
  return {
    sourceId: node.id,
    frame: frameOf(node),
    order,
    content: nodeContent(node),
    ...(appearance ? { appearance } : {}),
    ...(node.type === "file" && node.subpath ? { fileSubpath: node.subpath } : {}),
    ...(parentSectionSourceId ? { parentSectionSourceId } : {}),
  };
}

function frameOf(node: JsonCanvasNodeV1) {
  return { x: node.x, y: node.y, width: node.width, height: node.height };
}

function isGroup(node: JsonCanvasNodeV1): node is Extract<JsonCanvasNodeV1, { type: "group" }> {
  return node.type === "group";
}

function contains(parent: JsonCanvasNodeV1, child: JsonCanvasNodeV1) {
  return (
    child.x >= parent.x &&
    child.y >= parent.y &&
    child.x + child.width <= parent.x + parent.width &&
    child.y + child.height <= parent.y + parent.height
  );
}

function nodeContent(node: Exclude<JsonCanvasNodeV1, { type: "group" }>): JsonCanvasNodeContentV1 {
  if (node.type === "text") return { type: "text", text: node.text };
  if (node.type === "file") return { type: "file", file: node.file };
  return { type: "link", url: node.url };
}

function nodeAppearance(node: JsonCanvasNodeV1) {
  const fill = node.color ? canvasFillForJsonColor(node.color) : undefined;
  const appearance = {
    ...fill,
    ...(node.type === "group" && node.background ? { background: node.background } : {}),
    ...(node.type === "group" && node.backgroundStyle ? { backgroundStyle: node.backgroundStyle } : {}),
  };
  return Object.keys(appearance).length ? appearance : undefined;
}
