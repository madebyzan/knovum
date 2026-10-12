import { extname } from "pathe";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { DocumentFormat } from "../catalog/creation-intent";

const TARGET_FORMATS: readonly DocumentFormat[] = ["docx", "pdf", "txt", "html", "pptx"];
const SOURCE_FILE_FORMATS = new Set(["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"]);
const OFFICE_SOURCE_FORMATS = new Set(["doc", "docx", "xls", "xlsx", "ppt", "pptx"]);

export type DocumentConversionSource = SpaceCanvasNode & {
  data: { kind: "note" | "markdown" | "file"; title: string; knowledgeRevision: string };
};

function resolveSourceFormat(node: SpaceCanvasNode | undefined): string | undefined {
  if (!node?.data.knowledgeRevision) return;
  const content = node.data;
  if (content.kind === "note") return "note";
  if (content.kind === "markdown") return content.derivationState ? undefined : "markdown";
  if (content.kind !== "file" || content.processingState || !content.sourceRevision) return;
  const extension = extname(content.assetPath).slice(1).toLowerCase();
  return SOURCE_FILE_FORMATS.has(extension) ? extension : undefined;
}

export function isDocumentConversionSource(node: SpaceCanvasNode): node is DocumentConversionSource {
  return resolveSourceFormat(node) !== undefined;
}

export function getDocumentConversionTargets(node: SpaceCanvasNode | undefined): readonly DocumentFormat[] {
  const format = resolveSourceFormat(node);
  if (!format) return [];
  return TARGET_FORMATS.filter(
    (target) =>
      target !== format &&
      !(target === "pdf" && OFFICE_SOURCE_FORMATS.has(format)) &&
      (target !== "pptx" || ["note", "markdown", "pdf", "ppt"].includes(format)),
  );
}

export function needsDocumentRecognition(node: SpaceCanvasNode | undefined): boolean {
  if (resolveSourceFormat(node) !== "pdf" || node?.data.kind !== "file") return false;
  const coverage = node.data.documentCoverage;
  if (!coverage?.pagesRequiringRecognition.length) return false;
  if (coverage.sourceRevision !== node.data.sourceRevision) return true;
  return coverage.pagesRequiringRecognition.some((page) => {
    const recognition = coverage.occurrences.find(
      (occurrence) => occurrence.pdfInput?.kind === "page" && occurrence.pdfInput.page === page,
    );
    return recognition?.state !== "completed" || !recognition.analysis?.detectedText.trim();
  });
}
