import { basename } from "pathe";

import type { SpaceEditorDocument, CanvasNodeLayoutV1 } from "../canvas/contract/canvas-document";
import type { CanvasHistoryEntry, CanvasSurfaceEvents } from "../canvas/contract/canvas-editor";
import type { CanvasEditorPort } from "../canvas/contract/canvas-editor-port";
import { createCanvasPersistenceHistory } from "../canvas/persistence/canvas-persistence-history";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceRelation } from "../model/relations/relation";
import type {
  CanvasExchangeCodec,
  CanvasExchangeExport,
  CanvasExchangePort,
  CanvasExchangeResource,
} from "./canvas-exchange-port";
import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportDraft,
  JsonCanvasNodeContentV1,
} from "./json-canvas-import-contract";
import { exportJsonCanvasDocument, importJsonCanvasDocument } from "./json-canvas-mapping";

interface JsonCanvasExchangeOptions {
  codec: CanvasExchangeCodec;
  editor: CanvasEditorPort;
  nodes(): readonly SpaceCanvasNode[];
  relations(): readonly SpaceRelation[];
  commitImport(request: CanvasJsonImportDraft): Promise<CanvasJsonImportCommitResult>;
  historyEntry(entry: CanvasHistoryEntry): void;
  historyTransition: CanvasSurfaceEvents["onHistoryTransition"];
  historyTransaction(action: () => Promise<void>): Promise<void>;
}

export function createJsonCanvasExchangePort(options: JsonCanvasExchangeOptions): CanvasExchangePort {
  return {
    async importJsonCanvas(source, sourceDirectory) {
      const plan = importJsonCanvasDocument(await options.codec.parse(source));
      await options.historyTransaction(async () => {
        const before = options.editor.getDocument();
        const result = await options.commitImport({ sourceDirectory, plan, document: before });
        options.editor.adoptDocument(result.document);
        options.historyEntry(
          createCanvasPersistenceHistory({
            before,
            after: result.document,
            persistence: result.history,
            current: options.editor.getDocument,
            commit: options.historyTransition,
            adopt: options.editor.adoptDocument,
          }),
        );
      });
    },
    async exportJsonCanvas() {
      return exportDocument(
        options.editor.getDocument(),
        options.nodes(),
        options.relations(),
        options.codec,
      );
    },
  };
}

async function exportDocument(
  document: SpaceEditorDocument,
  nodes: readonly SpaceCanvasNode[],
  relations: readonly SpaceRelation[],
  codec: CanvasExchangeCodec,
): Promise<CanvasExchangeExport> {
  const content = new Map(nodes.map((node) => [node.id, node]));
  const resources: CanvasExchangeResource[] = [];
  const exportNodes = document.nodes.flatMap((layout) => {
    const node = content.get(layout.nodeId);
    if (!node) return [];
    const nodeContent = exportContent(node, resources);
    return [{ layout: exportLayout(layout, nodeContent, resources), content: nodeContent }];
  });
  return {
    source: await codec.serialize(exportJsonCanvasDocument(exportNodes, relations)),
    resources,
  };
}

function exportLayout(
  layout: CanvasNodeLayoutV1,
  content: JsonCanvasNodeContentV1,
  resources: CanvasExchangeResource[],
) {
  const background = layout.appearance?.background;
  if (content.type !== "group" || !background) return layout;
  return {
    ...layout,
    appearance: {
      ...layout.appearance,
      background: registerResource(layout.nodeId, background, resources),
    },
  };
}

function exportContent(node: SpaceCanvasNode, resources: CanvasExchangeResource[]): JsonCanvasNodeContentV1 {
  const data = node.data;
  if (data.kind === "section") return { type: "group", label: data.title };
  if (data.kind === "website") return { type: "link", url: data.url };
  if ("assetPath" in data) {
    return { type: "file", file: registerResource(node.id, data.assetPath, resources) };
  }
  return { type: "text", text: [data.title, nodeSummary(data)].filter(Boolean).join("\n\n") };
}

function registerResource(nodeId: string, sourcePath: string, resources: CanvasExchangeResource[]) {
  const relativePath = `assets/${nodeId}-${basename(sourcePath)}`;
  resources.push({ sourcePath, relativePath });
  return relativePath;
}

function nodeSummary(data: SpaceNodePayload) {
  return textNodeSummary(data) ?? structuredNodeSummary(data);
}

function textNodeSummary(data: SpaceNodePayload) {
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

function structuredNodeSummary(data: SpaceNodePayload) {
  if ("fileName" in data) return data.fileName;
  if (data.kind === "credential") return data.credentialKind;
  if (data.kind === "object") return data.summary ?? data.objectTypeId;
  if (data.kind === "portal") return data.targetSceneId ? "Linked scene" : "Linked space";
  return "";
}
