import { invoke } from "@tauri-apps/api/core";

import { i18n } from "../../../../foundation/i18n/i18n";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload, SpaceMarkdownPayload } from "../../model/nodes/node-payload";
import type { useSpaceStore } from "../../session/space-store";
import { markProcessingNodeFailed } from "../pending-node";
import { derivedNodePosition } from "../placement";
import { runDerivation } from "../run-derivation";

type Store = ReturnType<typeof useSpaceStore>;
type DocumentFormat = "docx" | "pdf";
interface ExportOutput {
  nodeId: string;
  data: SpaceFilePayload;
}

interface ExportMarkdownDocumentOptions {
  store: Store;
  source: CurrentMarkdownNode;
  format: DocumentFormat;
}

type CurrentMarkdownNode = SpaceCanvasNode & {
  data: { kind: "markdown"; title: string; knowledgeRevision: string };
};

const formats = {
  docx: {
    pathCommand: "plugin:spaces|allocate_document_docx_path",
    exportCommand: "plugin:spaces|export_document_as_docx",
    taskKey: "spaces.creation.docxTask",
    failureKey: "spaces.creation.docxFailed",
  },
  pdf: {
    pathCommand: "plugin:spaces|allocate_document_pdf_path",
    exportCommand: "plugin:spaces|export_document_as_pdf",
    taskKey: "spaces.creation.pdfTask",
    failureKey: "spaces.creation.pdfFailed",
  },
} as const;

export async function exportMarkdownDocument({ store, source, format }: ExportMarkdownDocumentOptions) {
  if (!store.activeSpaceId) return;
  const descriptor = formats[format];
  const spaceId = store.activeSpaceId;
  const title = `${source.data.title}.${format}`;
  const assetPath = await invoke<string>(descriptor.pathCommand);
  const [output] = await store.addItem(
    { kind: "file", title, fileName: title, assetPath, processingState: "processing" },
    derivedNodePosition([source], useCanvasEditorSession().selection.value.nodePlacements),
    { reveal: true },
  );
  if (!output) throw new Error("Document output node was not created");

  return runDerivation<ExportOutput>({
    store,
    pendingNode: output,
    sourceNodeIds: [source.id],
    task: {
      title: i18n.global.t(descriptor.taskKey),
      detail: title,
    },
    failureMessageKey: descriptor.failureKey,
    execute: () =>
      invoke<ExportOutput>(descriptor.exportCommand, {
        request: {
          id: crypto.randomUUID(),
          spaceId,
          sourceNodeId: source.id,
          sourceRevision: source.data.knowledgeRevision,
          outputNodeId: output.id,
        },
      }),
    onSuccess: async (completed, currentSpaceId) => {
      store.applyNodeDataForSpace(currentSpaceId, completed.nodeId, completed.data);
      await store.refreshActiveSpace();
      store.requestReveal([completed.nodeId], "ensure-visible");
    },
    onError: async () => {
      await markProcessingNodeFailed(store, spaceId, output);
    },
  });
}

export type PopulatedMarkdownNode = SpaceCanvasNode & {
  data: SpaceMarkdownPayload & { text: string; knowledgeRevision: string };
};

export function isKnowledgeMarkdown(node: SpaceCanvasNode): node is CurrentMarkdownNode {
  return node.data.kind === "markdown" && Boolean(node.data.knowledgeRevision);
}

export function isPopulatedMarkdown(node: SpaceCanvasNode): node is PopulatedMarkdownNode {
  return isKnowledgeMarkdown(node) && Boolean(node.data.text);
}

export const isCurrentMarkdown = isKnowledgeMarkdown;
