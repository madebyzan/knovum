import {
  MAX_CANVAS_Z_INDEX,
  type CanvasDocumentV1,
  type CanvasNodeFrame,
  type CanvasNodeLayoutV1,
} from "../canvas/contract/canvas-document";
import type { SpaceRelation } from "../model/relations/relation";
import { canvasFillForJsonColor, jsonColorForCanvasFill } from "./json-canvas-colors";
import type { JsonCanvasDocumentV1, JsonCanvasNodeV1 } from "./json-canvas-contract";
import type {
  JsonCanvasImportNodeV1,
  JsonCanvasImportPlanV1,
  JsonCanvasNodeContentV1,
} from "./json-canvas-import-contract";

export interface JsonCanvasExportNodeV1 {
  layout: CanvasNodeLayoutV1;
  content: JsonCanvasNodeContentV1;
}

interface MergeImportedJsonCanvasOptions {
  plan: JsonCanvasImportPlanV1;
  importedNodes: readonly { sourceId: string; nodeId: string }[];
  backgroundPaths?: ReadonlyMap<string, string>;
}

export function exportJsonCanvasDocument(
  nodes: readonly JsonCanvasExportNodeV1[],
  relations: readonly SpaceRelation[],
): JsonCanvasDocumentV1 {
  const nodeIds = new Set(nodes.map(({ layout }) => layout.nodeId));
  return {
    nodes: nodes.toSorted((left, right) => left.layout.zIndex - right.layout.zIndex).map(exportNode),
    edges: relations.flatMap((relation) =>
      relation.status !== "invalidated" && nodeIds.has(relation.sourceId) && nodeIds.has(relation.targetId)
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

function exportNode({ layout, content }: JsonCanvasExportNodeV1): JsonCanvasNodeV1 {
  const node = { id: layout.nodeId, ...standardFrame(layout.frame), ...content } as JsonCanvasNodeV1;
  applyExportColor(node, layout);
  if (node.type === "group") applyGroupAppearance(node, layout);
  if (node.type === "file") applyFileSubpath(node, layout);
  return node;
}

function applyExportColor(node: JsonCanvasNodeV1, layout: CanvasNodeLayoutV1) {
  const color = layout.appearance?.customFill ?? layout.appearance?.fill;
  if (!color) return;
  node.color = layout.appearance?.customFill ? color : jsonColorForCanvasFill(layout.appearance!.fill!);
}

function applyGroupAppearance(
  node: Extract<JsonCanvasNodeV1, { type: "group" }>,
  layout: CanvasNodeLayoutV1,
) {
  const { background, backgroundStyle } = layout.appearance ?? {};
  if (background) node.background = background;
  if (backgroundStyle) node.backgroundStyle = backgroundStyle;
}

function applyFileSubpath(node: Extract<JsonCanvasNodeV1, { type: "file" }>, layout: CanvasNodeLayoutV1) {
  if (layout.fileSubpath) node.subpath = layout.fileSubpath;
}

function standardFrame(frame: CanvasNodeFrame) {
  const geometry = {
    x: Math.round(frame.x),
    y: Math.round(frame.y),
    width: Math.round(frame.width),
    height: Math.round(frame.height),
  };
  if (!Object.values(geometry).every(Number.isSafeInteger) || geometry.width <= 0 || geometry.height <= 0) {
    throw new Error("Canvas frame cannot be represented by JSON Canvas 1.0 integer geometry");
  }
  return geometry;
}

export function mergeImportedJsonCanvasDocument(
  document: CanvasDocumentV1,
  options: MergeImportedJsonCanvasOptions,
): CanvasDocumentV1 {
  const ids = new Map(options.importedNodes.map(({ sourceId, nodeId }) => [sourceId, nodeId]));
  const zOffset = document.nodes.reduce((maximum, node) => Math.max(maximum, node.zIndex + 1), 0);
  const nodes = options.plan.nodes.map((node) => {
    const zIndex = node.zIndex + zOffset;
    const background = options.backgroundPaths?.get(node.sourceId);
    if (zIndex > MAX_CANVAS_Z_INDEX) throw new Error("Imported canvas exceeds the supported layer range");
    return {
      nodeId: importedNodeId(ids, node.sourceId),
      frame: { ...node.frame },
      sizeAuthority: "frame" as const,
      ...(node.appearance
        ? {
            appearance: {
              ...node.appearance,
              ...(background ? { background } : {}),
            },
          }
        : {}),
      ...(node.fileSubpath ? { fileSubpath: node.fileSubpath } : {}),
      zIndex,
    };
  });
  return {
    ...document,
    nodes: [...document.nodes, ...nodes],
  };
}

export function importJsonCanvasDocument(document: JsonCanvasDocumentV1): JsonCanvasImportPlanV1 {
  const nodes = document.nodes ?? [];
  return {
    nodes: nodes.map((node, zIndex) => importNode(node, zIndex, nodes)),
    edges: document.edges ?? [],
  };
}

function importNode(node: JsonCanvasNodeV1, zIndex: number, nodes: readonly JsonCanvasNodeV1[]) {
  const imported: JsonCanvasImportNodeV1 = {
    sourceId: node.id,
    frame: { x: node.x, y: node.y, width: node.width, height: node.height, rotation: 0 },
    zIndex,
    content: nodeContent(node),
  };
  const appearance = nodeAppearance(node);
  const parentSourceId = containingGroupId(node, nodes);
  if (appearance) imported.appearance = appearance;
  if (node.type === "file" && node.subpath) imported.fileSubpath = node.subpath;
  if (parentSourceId) imported.parentSourceId = parentSourceId;
  return imported;
}

function containingGroupId(node: JsonCanvasNodeV1, nodes: readonly JsonCanvasNodeV1[]) {
  if (node.type === "group") return undefined;
  return nodes
    .filter(
      (candidate): candidate is Extract<JsonCanvasNodeV1, { type: "group" }> =>
        candidate.type === "group" && contains(candidate, node),
    )
    .toSorted(compareContainingGroups)[0]?.id;
}

function contains(parent: JsonCanvasNodeV1, child: JsonCanvasNodeV1) {
  return (
    child.x >= parent.x &&
    child.y >= parent.y &&
    child.x + child.width <= parent.x + parent.width &&
    child.y + child.height <= parent.y + parent.height
  );
}

function importedNodeId(nodeIds: ReadonlyMap<string, string>, sourceId: string) {
  const nodeId = nodeIds.get(sourceId);
  if (!nodeId) throw new Error(`Imported node ${sourceId} is unavailable`);
  return nodeId;
}

function compareContainingGroups(left: JsonCanvasNodeV1, right: JsonCanvasNodeV1) {
  const areaDifference = left.width * left.height - right.width * right.height;
  return areaDifference || left.id.localeCompare(right.id);
}

function nodeContent(node: JsonCanvasNodeV1): JsonCanvasNodeContentV1 {
  if (node.type === "text") return { type: "text", text: node.text };
  if (node.type === "file") return { type: "file", file: node.file };
  if (node.type === "link") return { type: "link", url: node.url };
  return { type: "group", ...(node.label ? { label: node.label } : {}) };
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
