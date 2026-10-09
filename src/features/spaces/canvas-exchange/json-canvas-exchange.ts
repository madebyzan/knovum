import { compact } from "es-toolkit";
import { basename } from "pathe";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { projectStructureScene } from "../canvas/document/structure-scene-projection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { describeSpaceNode } from "../model/nodes/node-description";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import type { WorldBounds } from "../model/structure/world-bounds";
import type {
  CanvasExchangeExport,
  CanvasExchangePort,
  CanvasExchangeResource,
} from "./canvas-exchange-port";
import {
  assertJsonCanvasExportable,
  assertSectionMembershipIsGeometric,
} from "./json-canvas-export-validation";
import type {
  CanvasImportReceipt,
  CanvasJsonImportDraft,
  JsonCanvasNodeContentV1,
} from "./json-canvas-import-contract";
import { exportJsonCanvasDocument, type JsonCanvasExportNodeV1 } from "./json-canvas-mapping";

interface JsonCanvasExchangeOptions {
  snapshot(): SpaceEditSnapshot;
  nodes(): readonly SpaceCanvasNode[];
  relations(): readonly SpaceRelation[];
  commitImport(request: CanvasJsonImportDraft): Promise<CanvasImportReceipt>;
  transaction(action: () => Promise<void>): Promise<void>;
}

export function createJsonCanvasExchangePort(options: JsonCanvasExchangeOptions): CanvasExchangePort {
  return {
    async importJsonCanvas(sourcePath) {
      try {
        await options.transaction(async () => void (await options.commitImport({ sourcePath })));
      } catch (error) {
        if (decodeAppError(error)?.code !== "json_canvas_import_cancelled") throw error;
      }
    },
    exportJsonCanvas: () => exportDocument(options.snapshot(), options.nodes(), options.relations()),
  };
}

async function exportDocument(
  snapshot: SpaceEditSnapshot,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
): Promise<CanvasExchangeExport> {
  assertJsonCanvasExportable(snapshot, relations);
  const scene = projectStructureScene({ snapshot, nodes, relations });
  assertSectionMembershipIsGeometric(scene.content, scene.sections);
  const resources: CanvasExchangeResource[] = [];
  const sectionByStructureNodeId = new Map(
    scene.sections.map(({ section, layout, bounds }) => [
      section.id,
      {
        id: section.id,
        frame: frame(bounds),
        appearance: packagedAppearance(section.id, layout.appearance, resources),
        content: { type: "group" as const, label: section.title },
      },
    ]),
  );
  const contentByStructureNodeId = new Map(
    scene.content.map(({ structureNode, bounds, layout, node }) => [
      structureNode.id,
      {
        id: node.id,
        frame: frame(bounds),
        appearance: layout.appearance,
        ...(layout.fileSubpath ? { fileSubpath: layout.fileSubpath } : {}),
        content: exportContent(node, resources),
      },
    ]),
  );
  const exportNodes = orderedExportNodes(snapshot, sectionByStructureNodeId, contentByStructureNodeId);
  return {
    document: exportJsonCanvasDocument(exportNodes, relations),
    resources,
  };
}

function orderedExportNodes(
  snapshot: SpaceEditSnapshot,
  sectionByStructureNodeId: ReadonlyMap<string, JsonCanvasExportNodeV1>,
  contentByStructureNodeId: ReadonlyMap<string, JsonCanvasExportNodeV1>,
) {
  const childrenByParentId = new Map<string | null, SpaceEditSnapshot["structure"][number][]>();
  for (const structureNode of snapshot.structure) {
    const siblings = childrenByParentId.get(structureNode.parentId) ?? [];
    childrenByParentId.set(structureNode.parentId, [...siblings, structureNode]);
  }
  for (const siblings of childrenByParentId.values())
    siblings.sort((left, right) => left.order - right.order);
  const ordered: JsonCanvasExportNodeV1[] = [];
  function append(parentId: string | null) {
    for (const structureNode of childrenByParentId.get(parentId) ?? []) {
      const node =
        structureNode.kind === "section"
          ? sectionByStructureNodeId.get(structureNode.id)
          : contentByStructureNodeId.get(structureNode.id);
      if (!node) throw new Error(`JSON Canvas export node is unavailable: ${structureNode.id}`);
      ordered.push(node);
      append(structureNode.id);
    }
  }
  append(null);
  return ordered;
}

function frame(bounds: WorldBounds) {
  return {
    x: bounds.left,
    y: bounds.top,
    width: bounds.right - bounds.left,
    height: bounds.bottom - bounds.top,
  };
}

function packagedAppearance(
  structureNodeId: string,
  appearance: JsonCanvasExportNodeV1["appearance"],
  resources: CanvasExchangeResource[],
) {
  if (!appearance?.background) return appearance;
  return {
    ...appearance,
    background: registerResource(structureNodeId, appearance.background, resources),
  };
}

function exportContent(node: SpaceCanvasNode, resources: CanvasExchangeResource[]): JsonCanvasNodeContentV1 {
  const data = node.data;
  if (data.kind === "website") return { type: "link", url: data.url };
  if ("assetPath" in data)
    return { type: "file", file: registerResource(node.id, data.assetPath, resources) };
  return { type: "text", text: compact([data.title, describeSpaceNode(data)]).join("\n\n") };
}

function registerResource(nodeId: string, sourcePath: string, resources: CanvasExchangeResource[]) {
  const relativePath = `assets/${nodeId}-${basename(sourcePath)}`;
  resources.push({ sourcePath, relativePath });
  return relativePath;
}
