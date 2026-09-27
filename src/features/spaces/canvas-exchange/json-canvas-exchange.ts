import { compact } from "es-toolkit";
import { basename } from "pathe";

import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import { projectStructureScene } from "../canvas/document/structure-scene-projection";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type {
  CanvasExchangeCodec,
  CanvasExchangeExport,
  CanvasExchangePort,
  CanvasExchangeResource,
} from "./canvas-exchange-port";
import {
  assertJsonCanvasExportable,
  assertSectionMembershipIsGeometric,
} from "./json-canvas-export-validation";
import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportDraft,
  JsonCanvasNodeContentV1,
} from "./json-canvas-import-contract";
import {
  exportJsonCanvasDocument,
  importJsonCanvasDocument,
  type JsonCanvasExportNodeV1,
} from "./json-canvas-mapping";

interface JsonCanvasExchangeOptions {
  codec: CanvasExchangeCodec;
  snapshot(): SpaceEditSnapshot;
  nodes(): readonly SpaceCanvasNode[];
  relations(): readonly SpaceRelation[];
  commitImport(request: CanvasJsonImportDraft): Promise<CanvasJsonImportCommitResult>;
  transaction(action: () => Promise<void>): Promise<void>;
}

export function createJsonCanvasExchangePort(options: JsonCanvasExchangeOptions): CanvasExchangePort {
  return {
    async importJsonCanvas(source, sourceDirectory) {
      const plan = importJsonCanvasDocument(await options.codec.parse(source));
      await options.transaction(async () => void (await options.commitImport({ sourceDirectory, plan })));
    },
    exportJsonCanvas: () =>
      exportDocument(options.snapshot(), options.nodes(), options.relations(), options.codec),
  };
}

async function exportDocument(
  snapshot: SpaceEditSnapshot,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
  codec: CanvasExchangeCodec,
): Promise<CanvasExchangeExport> {
  assertJsonCanvasExportable(snapshot, relations);
  const scene = projectStructureScene(snapshot, nodes, relations);
  assertSectionMembershipIsGeometric(scene.content, scene.sections);
  const resources: CanvasExchangeResource[] = [];
  const sections = new Map(
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
  const contents = new Map(
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
  const exportNodes = orderedExportNodes(snapshot, sections, contents);
  return {
    source: await codec.serialize(exportJsonCanvasDocument(exportNodes, relations)),
    resources,
  };
}

function orderedExportNodes(
  snapshot: SpaceEditSnapshot,
  sections: ReadonlyMap<string, JsonCanvasExportNodeV1>,
  contents: ReadonlyMap<string, JsonCanvasExportNodeV1>,
) {
  const children = new Map<string | null, SpaceEditSnapshot["structure"][number][]>();
  for (const structureNode of snapshot.structure) {
    const siblings = children.get(structureNode.parentId) ?? [];
    children.set(structureNode.parentId, [...siblings, structureNode]);
  }
  for (const siblings of children.values()) siblings.sort((left, right) => left.order - right.order);
  const ordered: JsonCanvasExportNodeV1[] = [];
  function append(parentId: string | null) {
    for (const structureNode of children.get(parentId) ?? []) {
      const node =
        structureNode.kind === "section" ? sections.get(structureNode.id) : contents.get(structureNode.id);
      if (!node) throw new Error(`JSON Canvas export node is unavailable: ${structureNode.id}`);
      ordered.push(node);
      append(structureNode.id);
    }
  }
  append(null);
  return ordered;
}

type Bounds = { left: number; top: number; right: number; bottom: number };

function frame(bounds: Bounds) {
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
  return { type: "text", text: compact([data.title, nodeSummary(data)]).join("\n\n") };
}

function registerResource(nodeId: string, sourcePath: string, resources: CanvasExchangeResource[]) {
  const relativePath = `assets/${nodeId}-${basename(sourcePath)}`;
  resources.push({ sourcePath, relativePath });
  return relativePath;
}

function nodeSummary(data: SpaceNodePayload) {
  const textual = textualSummary(data);
  if (textual !== undefined) return textual;
  if ("fileName" in data) return data.fileName;
  return metadataSummary(data);
}

function textualSummary(data: SpaceNodePayload): string | undefined {
  if (data.kind === "note" || data.kind === "text") return data.plainText;
  if (data.kind === "markdown") return data.text;
  if (data.kind === "website") return data.description ?? data.contentPreview ?? data.url;
  if (data.kind === "subtitle")
    return data.cues
      .slice(0, 3)
      .map(({ text }) => text)
      .join("\n");
  return undefined;
}

function metadataSummary(data: SpaceNodePayload) {
  if (data.kind === "credential") return data.credentialKind;
  if (data.kind === "object") return data.summary ?? data.objectTypeId;
  if (data.kind === "portal") return data.targetSceneId ? "Linked scene" : "Linked space";
  return "";
}
