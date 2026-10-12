import { describe, expect, it } from "vitest";

import { creationIntentCatalog } from "../knowledge-derivations/catalog/creation-intent-catalog";
import type { DocumentCoverage } from "../model/nodes/schema/document-coverage-fields";
import { documentFile } from "./support/document-conversion-fixture";

function recognitionCoverage(
  sourceRevision: string,
  state: "pending" | "completed",
  detectedText = "Recognized text",
): DocumentCoverage {
  const completed = state === "completed";
  const resourceId = "a".repeat(64);
  return {
    schemaVersion: 1,
    parser: "pdf-inspector",
    sourceRevision,
    pageCount: 1,
    pagesRequiringRecognition: [1],
    pagesWithExtractedText: [],
    pagesWithoutExtractableContent: [],
    resources: completed
      ? [
          {
            id: resourceId,
            path: "/assets/page.png",
            mediaType: "image/png",
            width: 800,
            height: 600,
            byteSize: 2048,
          },
        ]
      : [],
    occurrences: [
      {
        id: "pdf:1",
        location: { kind: "pdf", page: 1 },
        context: "",
        altText: null,
        resourceId: completed ? resourceId : null,
        pdfInput: { kind: "page", page: 1, reason: "scanned" },
        state,
        error: null,
        analysis: completed
          ? {
              description: "Scanned page",
              detectedText,
              executionSource: { type: "model", provider: "test", model: "vision" },
              generatedAt: 1,
              analysisVersion: "document-visual:1",
            }
          : null,
      },
    ],
    warnings: [],
  };
}

function conversionDecision(
  coverage: DocumentCoverage,
  action: "convert-document-to-docx" | "convert-document-to-html",
) {
  const source = documentFile("pdf");
  if (source.data.kind !== "file") throw new Error("File fixture expected");
  source.data.documentCoverage = coverage;
  return creationIntentCatalog([source]).decisions[action];
}

describe("PDF recognition conversion eligibility", () => {
  it.each([
    ["sha256:current", "pending", undefined],
    ["sha256:stale", "completed", undefined],
    ["sha256:current", "completed", "  "],
  ] as const)("blocks conversion while %s recognition is %s", (revision, state, text) => {
    expect(
      conversionDecision(recognitionCoverage(revision, state, text), "convert-document-to-docx"),
    ).toEqual({
      state: "blocked",
      reason: "needs-document-recognition",
    });
  });

  it("blocks coverage without the required page occurrence", () => {
    const coverage = recognitionCoverage("sha256:current", "completed");
    coverage.occurrences = [];
    coverage.resources = [];
    expect(conversionDecision(coverage, "convert-document-to-docx")).toEqual({
      state: "blocked",
      reason: "needs-document-recognition",
    });
  });

  it("blocks a completed occurrence without analysis", () => {
    const coverage = recognitionCoverage("sha256:current", "completed");
    coverage.occurrences[0]!.analysis = null;
    expect(conversionDecision(coverage, "convert-document-to-html")).toEqual({
      state: "blocked",
      reason: "needs-document-recognition",
    });
  });

  it("offers conversion after current recognition completes", () => {
    const coverage = recognitionCoverage("sha256:current", "completed");
    expect(conversionDecision(coverage, "convert-document-to-docx").state).toBe("ready");
    expect(conversionDecision(coverage, "convert-document-to-html").state).toBe("ready");
  });
});
