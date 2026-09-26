import { sumBy } from "es-toolkit";
import type { z } from "zod";

import type { DocumentCoverage } from "./document-coverage-fields";
import { validateDocumentInput } from "./document-input-validation";

export function validateDocumentCoverage(report: DocumentCoverage, context: z.RefinementCtx) {
  const ids = new Set(report.resources.map(({ id }) => id));
  const occurrences = new Set(report.occurrences.map(({ id }) => id));
  if (ids.size !== report.resources.length || occurrences.size !== report.occurrences.length) {
    context.addIssue({ code: "custom", message: "Document resource and occurrence IDs must be unique" });
  }
  if ((report.parser === "pdf-inspector") !== (report.pageCount !== null)) {
    context.addIssue({ code: "custom", message: "Document parser and page coverage disagree" });
  }
  validatePages(report, context);
  validateBudgets(report, context);
  for (const occurrence of report.occurrences) {
    validateDocumentInput(occurrence, report, context);
    validateCompletion(occurrence, ids, context);
  }
  const referencedIds = new Set(
    report.occurrences.flatMap(({ resourceId }) => (resourceId === null ? [] : [resourceId])),
  );
  if (ids.size !== referencedIds.size || [...ids].some((id) => !referencedIds.has(id))) {
    context.addIssue({ code: "custom", message: "Document resources must belong to an occurrence" });
  }
}

function validatePages(report: DocumentCoverage, context: z.RefinementCtx) {
  const claimed = new Set<number>();
  for (const pages of [
    report.pagesRequiringRecognition,
    report.pagesWithExtractedText,
    report.pagesWithoutExtractableContent,
  ]) {
    for (const page of pages) {
      if (report.pageCount === null || page > report.pageCount || claimed.has(page)) {
        context.addIssue({
          code: "custom",
          message: "PDF coverage pages must be unique, in range and non-overlapping",
        });
      }
      claimed.add(page);
    }
  }
  validateRecognitionInputs(report, context);
}

function validateRecognitionInputs(report: DocumentCoverage, context: z.RefinementCtx) {
  const pageInputs = report.occurrences.flatMap(({ pdfInput }) =>
    pdfInput?.kind === "page" ? [pdfInput.page] : [],
  );
  const recognition = new Set(report.pagesRequiringRecognition);
  if (
    pageInputs.length !== recognition.size ||
    new Set(pageInputs).size !== pageInputs.length ||
    pageInputs.some((page) => !recognition.has(page))
  ) {
    context.addIssue({ code: "custom", message: "PDF recognition coverage and page inputs disagree" });
  }
}

function validateCompletion(
  occurrence: DocumentCoverage["occurrences"][number],
  ids: Set<string>,
  context: z.RefinementCtx,
) {
  if (occurrence.resourceId && !ids.has(occurrence.resourceId)) {
    context.addIssue({ code: "custom", message: "Document occurrence references an unavailable resource" });
  }
  if ((occurrence.state === "completed") !== (occurrence.analysis !== null)) {
    context.addIssue({ code: "custom", message: "Document completion requires an analysis" });
  }
  if (occurrence.state === "completed" && !occurrence.resourceId) {
    context.addIssue({ code: "custom", message: "Completed document analysis requires its durable raster" });
  }
}

function validateBudgets(report: DocumentCoverage, context: z.RefinementCtx) {
  if (sumBy(report.resources, ({ byteSize }) => byteSize) > 256 * 1024 * 1024) {
    context.addIssue({ code: "custom", message: "Document raster resources exceed 256 MiB" });
  }
  const units = sumBy(
    report.occurrences,
    ({ analysis }) => (analysis?.description.length ?? 0) + (analysis?.detectedText.length ?? 0),
  );
  if (units > 4 * 1024 * 1024) {
    context.addIssue({ code: "custom", message: "Document visual text exceeds 4 Mi UTF-16 units" });
  }
}
