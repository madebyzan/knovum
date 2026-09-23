import { describe, expect, it } from "vitest";

import { documentCoverageSchema } from "../model/nodes/schema/document-coverage";
import type { DocumentCoverage } from "../model/nodes/schema/document-coverage-fields";
import { projectDocumentResources } from "../presentation/document/document-resource-projection";

function report(parser: DocumentCoverage["parser"]): DocumentCoverage {
  return {
    schemaVersion: 1,
    parser,
    sourceRevision: "revision-1",
    pageCount: parser === "pdf-inspector" ? 3 : null,
    pagesRequiringRecognition: [],
    pagesWithExtractedText: parser === "pdf-inspector" ? [1, 2, 3] : [],
    pagesWithoutExtractableContent: [],
    resources: [
      {
        id: "a".repeat(64),
        path: "/assets/document-a.png",
        mediaType: "image/png",
        width: 1200,
        height: 800,
        byteSize: 4096,
      },
    ],
    occurrences: [
      {
        id: "visual-1",
        location:
          parser === "pdf-inspector"
            ? { kind: "pdf", page: 2 }
            : { kind: "word", section: 1, element: 4, headingPath: ["Results"] },
        context: "Nearby document text",
        altText: null,
        resourceId: "a".repeat(64),
        pdfInput:
          parser === "pdf-inspector"
            ? { kind: "image", page: 2, rect: { x: 10, y: 20, width: 300, height: 200 } }
            : null,
        state: "pending",
        error: null,
        analysis: null,
      },
      {
        id: "visual-2",
        location:
          parser === "pdf-inspector"
            ? { kind: "pdf", page: 3 }
            : { kind: "word", section: 2, element: 1, headingPath: [] },
        context: "The same image is reused",
        altText: "Diagram",
        resourceId: "a".repeat(64),
        pdfInput:
          parser === "pdf-inspector"
            ? { kind: "image", page: 3, rect: { x: 30, y: 40, width: 300, height: 200 } }
            : null,
        state: "pending",
        error: null,
        analysis: null,
      },
    ],
    warnings: parser === "pdf-inspector" ? ["pdf_visual_coverage_limited"] : [],
  };
}

describe("document resource projection", () => {
  it("groups repeated Office occurrences around the extracted image resource", () => {
    const [resource] = projectDocumentResources(report("office-oxide"));

    expect(resource?.sourceKind).toBe("embedded");
    expect(resource?.locationLabel).toBe("Word · Results");
    expect(resource?.occurrences.map(({ id }) => id)).toEqual(["visual-1", "visual-2"]);
  });

  it("labels PDF visual regions as rendered resources", () => {
    const [resource] = projectDocumentResources(report("pdf-inspector"));

    expect(resource?.sourceKind).toBe("rendered");
    expect(resource?.locationLabel).toBe("PDF · p.2");
    expect(resource?.occurrences).toHaveLength(2);
  });

  it("rejects resources that do not belong to a document occurrence", () => {
    const orphaned = report("office-oxide");
    orphaned.occurrences = [];

    expect(documentCoverageSchema.safeParse(orphaned).success).toBe(false);
  });
});
