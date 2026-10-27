import { describe, expect, it } from "vitest";

import { attachmentFileType } from "../conversation-attachment-presentation";

describe("attachment file type", () => {
  it("names a document by the type its file name carries", () => {
    expect(attachmentFileType("Git Branch Specification.pptx")).toBe("PPTX");
    expect(attachmentFileType("report.final.PDF ")).toBe("PDF");
    expect(attachmentFileType("take.mp3")).toBe("MP3");
  });

  it("names nothing for a title without a file type", () => {
    // A canvas node is titled by what it says, not by a file name; a sentence with a full stop
    // or a dotted version is not a type either.
    expect(attachmentFileType("Quarterly review")).toBeUndefined();
    expect(attachmentFileType("Notes on v2.")).toBeUndefined();
    expect(attachmentFileType("Release 1.2")).toBeUndefined();
    expect(attachmentFileType("Draft.toolongext")).toBeUndefined();
  });
});
