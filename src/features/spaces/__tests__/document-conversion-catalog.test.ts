import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { creationIntentCatalog } from "../knowledge-derivations/catalog/creation-intent-catalog";
import type { SpaceFilePayload } from "../model/nodes/node-payload";

function file(extension: string) {
  return createSpaceNode<SpaceFilePayload>(
    {
      kind: "file",
      title: "Report",
      fileName: `report.${extension}`,
      assetPath: `/spaces/assets/report.${extension}`,
      sourceRevision: "sha256:current",
      knowledgeRevision: "file:current",
    },
    { x: 0, y: 0 },
    "file",
  );
}

describe("document conversion selection", () => {
  it.each(["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "DOCX"])(
    "offers content conversion for a current %s without cached extracted text",
    (extension) => {
      const decisions = creationIntentCatalog([file(extension)]).decisions;
      expect(decisions["convert-document-to-txt"].state).toBe("ready");
      expect(decisions["convert-document-to-html"].state).toBe("ready");
    },
  );

  it.each([
    ["pdf", "convert-document-to-pdf"],
    ["docx", "convert-document-to-docx"],
    ["pptx", "render-presentation"],
  ] as const)("hides the existing %s format", (extension, action) => {
    expect(creationIntentCatalog([file(extension)]).decisions[action].state).toBe("hidden");
  });

  it("converts one PDF to Word and one Word document to PDF", () => {
    expect(creationIntentCatalog([file("pdf")]).decisions["convert-document-to-docx"].state).toBe("ready");
    expect(creationIntentCatalog([file("docx")]).decisions["convert-document-to-pdf"].state).toBe("ready");
  });

  it("uses the source asset format and exposes only supported slide conversions", () => {
    const source = file("pdf");
    if (source.data.kind !== "file") throw new Error("File fixture expected");
    source.data.fileName = "renamed.docx";
    const decisions = creationIntentCatalog([source]).decisions;
    expect(decisions["convert-document-to-pdf"].state).toBe("hidden");
    expect(decisions["convert-document-to-docx"].state).toBe("ready");
    expect(decisions["render-presentation"].state).toBe("ready");
    expect(creationIntentCatalog([file("ppt")]).decisions["render-presentation"].state).toBe("ready");
    expect(creationIntentCatalog([file("docx")]).decisions["render-presentation"].state).toBe("hidden");
  });

  it("requires one supported, published source", () => {
    const document = file("pdf");
    expect(creationIntentCatalog([document, file("docx")]).decisions["convert-document-to-txt"].state).toBe(
      "hidden",
    );
    expect(creationIntentCatalog([file("bin")]).decisions["convert-document-to-txt"].state).toBe("hidden");
    if (document.data.kind !== "file") throw new Error("File fixture expected");
    document.data.processingState = "processing";
    expect(creationIntentCatalog([document]).decisions["convert-document-to-txt"].state).toBe("hidden");
    delete document.data.processingState;
    delete document.data.sourceRevision;
    expect(creationIntentCatalog([document]).decisions["convert-document-to-txt"].state).toBe("hidden");
  });
});
