import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { creationIntentCatalog } from "../knowledge-derivations/catalog/creation-intent-catalog";
import { documentFile } from "./support/document-conversion-fixture";

describe("document conversion selection", () => {
  it("offers every target for a current note", () => {
    const note = createSpaceNode(
      {
        kind: "note",
        title: "Notes",
        plainText: "Body",
        tiptapContent: { type: "doc" },
        knowledgeRevision: "note:current",
      },
      { x: 0, y: 0 },
      "note",
    );
    const decisions = creationIntentCatalog([note]).decisions;
    expect([
      decisions["convert-document-to-docx"].state,
      decisions["convert-document-to-pdf"].state,
      decisions["convert-document-to-txt"].state,
      decisions["convert-document-to-html"].state,
      decisions["render-presentation"].state,
    ]).toEqual(["ready", "ready", "ready", "ready", "ready"]);
  });

  it.each(["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "DOCX"])(
    "offers content conversion for a current %s without cached extracted text",
    (extension) => {
      const decisions = creationIntentCatalog([documentFile(extension)]).decisions;
      expect(decisions["convert-document-to-txt"].state).toBe("ready");
      expect(decisions["convert-document-to-html"].state).toBe("ready");
    },
  );

  it.each([
    ["pdf", "convert-document-to-pdf"],
    ["docx", "convert-document-to-docx"],
    ["pptx", "render-presentation"],
  ] as const)("hides the existing %s format", (extension, action) => {
    expect(creationIntentCatalog([documentFile(extension)]).decisions[action].state).toBe("hidden");
  });

  it("keeps Markdown and PDF content conversions but never offers Office-to-PDF", () => {
    const markdown = createSpaceNode(
      {
        kind: "markdown",
        title: "Draft",
        text: "Body",
        knowledgeRevision: "markdown:current",
      },
      { x: 0, y: 0 },
      "markdown",
    );
    expect(creationIntentCatalog([markdown]).decisions["convert-document-to-pdf"].state).toBe("ready");
    expect(creationIntentCatalog([documentFile("pdf")]).decisions["convert-document-to-docx"].state).toBe(
      "ready",
    );
    for (const extension of ["doc", "docx", "xls", "xlsx", "ppt", "pptx"]) {
      expect(
        creationIntentCatalog([documentFile(extension)]).decisions["convert-document-to-pdf"].state,
      ).toBe("hidden");
    }
  });

  it("uses the source asset format and exposes only supported slide conversions", () => {
    const source = documentFile("pdf");
    if (source.data.kind !== "file") throw new Error("File fixture expected");
    source.data.fileName = "renamed.docx";
    const decisions = creationIntentCatalog([source]).decisions;
    expect(decisions["convert-document-to-pdf"].state).toBe("hidden");
    expect(decisions["convert-document-to-docx"].state).toBe("ready");
    expect(decisions["render-presentation"].state).toBe("ready");
    expect(creationIntentCatalog([documentFile("ppt")]).decisions["render-presentation"].state).toBe("ready");
    expect(creationIntentCatalog([documentFile("docx")]).decisions["render-presentation"].state).toBe(
      "hidden",
    );
  });

  it("requires one supported, published source", () => {
    const document = documentFile("pdf");
    expect(
      creationIntentCatalog([document, documentFile("docx")]).decisions["convert-document-to-txt"].state,
    ).toBe("hidden");
    expect(creationIntentCatalog([documentFile("bin")]).decisions["convert-document-to-txt"].state).toBe(
      "hidden",
    );
    if (document.data.kind !== "file") throw new Error("File fixture expected");
    document.data.processingState = "processing";
    expect(creationIntentCatalog([document]).decisions["convert-document-to-txt"].state).toBe("hidden");
    delete document.data.processingState;
    delete document.data.sourceRevision;
    expect(creationIntentCatalog([document]).decisions["convert-document-to-txt"].state).toBe("hidden");
  });
});
