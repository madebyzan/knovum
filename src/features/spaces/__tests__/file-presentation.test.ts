import { describe, expect, it } from "vitest";

import { classifyFilePresentation, fileExtension, usesDocumentPreview } from "../assets/file-presentation";

describe("space file presentation", () => {
  it("normalizes extensions before choosing a presentation", () => {
    expect(fileExtension("Quarterly.Report.PDF")).toBe("pdf");
    expect(classifyFilePresentation("brief.DOCX")).toBe("document");
    expect(classifyFilePresentation("old-format.doc")).toBe("document");
    expect(classifyFilePresentation("sheet.XLSX")).toBe("spreadsheet");
    expect(classifyFilePresentation("data.CSV")).toBe("tabular-data");
    expect(classifyFilePresentation("deck.PPTX")).toBe("presentation");
    expect(classifyFilePresentation("notes.odt")).toBe("generic");
  });

  it("identifies presentations rendered by the document preview", () => {
    expect(usesDocumentPreview("pdf")).toBe(true);
    expect(usesDocumentPreview("document")).toBe(true);
    expect(usesDocumentPreview("spreadsheet")).toBe(true);
    expect(usesDocumentPreview("presentation")).toBe(true);
    expect(usesDocumentPreview("tabular-data")).toBe(false);
    expect(usesDocumentPreview("generic")).toBe(false);
  });
});
