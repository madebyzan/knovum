import { i18n } from "../../../../foundation/i18n/i18n";
import { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { spaceNodeDefaultDimensions } from "../../canvas/node-size";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload } from "../../model/nodes/node-payload";
import type { useSpaceStore } from "../../session/space-store";
import { derivedNodePosition } from "../placement";
import { runDerivation } from "../run-derivation";

type Store = ReturnType<typeof useSpaceStore>;
export type DocumentFormat = "docx" | "pdf" | "txt";
interface ExportOutput {
  nodeId: string;
  data: SpaceFilePayload;
}

interface ExportMarkdownDocumentOptions {
  store: Store;
  source: IndexedMarkdownNode;
  format: DocumentFormat;
}

type IndexedMarkdownNode = SpaceCanvasNode & {
  data: { kind: "markdown"; title: string; knowledgeRevision: string };
};

const formats = {
  docx: {
    exportCommand: "plugin:spaces|export_document_as_docx",
    taskKey: "spaces.creation.document.export.docxTask",
    failureKey: "spaces.creation.document.export.docxFailed",
  },
  pdf: {
    exportCommand: "plugin:spaces|export_document_as_pdf",
    taskKey: "spaces.creation.document.export.pdfTask",
    failureKey: "spaces.creation.document.export.pdfFailed",
  },
  txt: {
    exportCommand: "plugin:spaces|export_document_as_txt",
    taskKey: "spaces.creation.document.export.txtTask",
    failureKey: "spaces.creation.document.export.txtFailed",
  },
} as const;

export async function exportMarkdownDocument({ store, source, format }: ExportMarkdownDocumentOptions) {
  if (!store.activeSpaceId) return;
  const descriptor = formats[format];
  const spaceId = store.activeSpaceId;
  const title = `${source.data.title}.${format}`;
  const outputId = crypto.randomUUID();
  const expectedEditRevision = store.editSnapshot.editRevision;
  const targetParentId = store.groupScopePath.at(-1) ?? null;
  const position = derivedNodePosition([source], useCanvasEditorSession().selection.value.nodePlacements);

  return runDerivation<ExportOutput>({
    store,
    pendingId: outputId,
    sourceNodeIds: [source.id],
    task: {
      title: i18n.global.t(descriptor.taskKey),
      statusMessage: title,
    },
    failureMessageKey: descriptor.failureKey,
    execute: () =>
      invokeCommand<ExportOutput>(descriptor.exportCommand, {
        request: {
          id: crypto.randomUUID(),
          spaceId,
          sourceNodeId: source.id,
          sourceRevision: source.data.knowledgeRevision,
          outputNodeId: outputId,
          expectedEditRevision,
          targetParentId,
          position,
          size: spaceNodeDefaultDimensions("file"),
        },
      }),
    onSuccess: async (completed, currentSpaceId) => {
      store.applyNodeContentForSpace(currentSpaceId, completed.nodeId, completed.data);
      if (store.activeSpaceId === currentSpaceId) {
        await store.refreshActiveSpace();
        store.requestReveal([completed.nodeId], "ensure-visible");
      }
    },
  });
}

export function isKnowledgeMarkdown(node: SpaceCanvasNode): node is IndexedMarkdownNode {
  return node.data.kind === "markdown" && Boolean(node.data.knowledgeRevision);
}
