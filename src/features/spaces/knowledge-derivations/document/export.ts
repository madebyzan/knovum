import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import { startDerivationAction } from "../start-action";
export type DocumentFormat = "docx" | "pdf" | "txt";
const taskLabels: Record<DocumentFormat, string> = {
  docx: "spaces.creation.document.export.docxTask",
  pdf: "spaces.creation.document.export.pdfTask",
  txt: "spaces.creation.document.export.txtTask",
};
type IndexedMarkdownNode = SpaceCanvasNode & {
  data: { kind: "markdown"; title: string; knowledgeRevision: string };
};

export function exportMarkdownDocument({
  store,
  source,
  format,
}: {
  store: ReturnType<typeof useSpaceStore>;
  source: IndexedMarkdownNode;
  format: DocumentFormat;
}) {
  return startDerivationAction({
    store,
    sources: [
      {
        nodeId: source.id,
        title: source.data.title,
        role: "primary",
        sourceRevision: source.data.knowledgeRevision,
      },
    ],
    actionId: `export-document-as-${format}`,
    parameters: {},
    title: i18n.global.t(taskLabels[format]),
  });
}
export function isKnowledgeMarkdown(node: SpaceCanvasNode): node is IndexedMarkdownNode {
  return node.data.kind === "markdown" && Boolean(node.data.knowledgeRevision);
}
