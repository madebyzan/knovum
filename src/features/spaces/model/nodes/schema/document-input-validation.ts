import type { z } from "zod";

import type { DocumentCoverage } from "./document-coverage-fields";

type Occurrence = DocumentCoverage["occurrences"][number];

export function validateDocumentInput(
  occurrence: Occurrence,
  report: DocumentCoverage,
  context: z.RefinementCtx,
) {
  const input = occurrence.pdfInput;
  if ((input !== null) !== (report.pageCount !== null)) {
    context.addIssue({ code: "custom", message: "Document visual input belongs to a different format" });
  }
  if (!input) {
    if (!occurrence.resourceId && occurrence.state !== "unsupported") {
      context.addIssue({ code: "custom", message: "Office visual occurrence requires a raster resource" });
    }
    return;
  }
  validatePdfInput(occurrence, input, report, context);
}

function validatePdfInput(
  occurrence: Occurrence,
  input: NonNullable<Occurrence["pdfInput"]>,
  report: DocumentCoverage,
  context: z.RefinementCtx,
) {
  const location = occurrence.location;
  if (
    report.pageCount === null ||
    input.page > report.pageCount ||
    location.kind !== "pdf" ||
    location.page !== input.page
  ) {
    context.addIssue({ code: "custom", message: "PDF visual input and location disagree" });
    return;
  }
  if (location.rects !== undefined) {
    context.addIssue({ code: "custom", message: "Document visual citations use page-level locations" });
  }
  if (input.kind === "page") {
    if (!report.pagesRequiringRecognition.includes(input.page)) {
      context.addIssue({ code: "custom", message: "PDF page has no recognition requirement" });
    }
  } else {
    validateImageCoverage(input.page, report, context);
  }
}

function validateImageCoverage(page: number, report: DocumentCoverage, context: z.RefinementCtx) {
  if (
    report.pagesRequiringRecognition.includes(page) ||
    report.pagesWithoutExtractableContent.includes(page)
  ) {
    context.addIssue({ code: "custom", message: "PDF page and region inputs must not duplicate coverage" });
  }
}
