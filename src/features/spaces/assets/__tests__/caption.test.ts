import { describe, expect, it } from "vitest";

import type { SpaceCodePayload, SpaceFilePayload } from "../../model/nodes/node-payload";
import { spaceNodeCaption } from "../caption";

const DOCX_BYTES = 2_726_298;

function file(fileName: string, size?: number): SpaceFilePayload {
  const data: SpaceFilePayload = {
    kind: "file",
    title: fileName,
    assetPath: `/spaces/assets/${fileName}`,
    fileName,
  };
  if (size !== undefined) data.size = size;
  return data;
}

describe("spaces node caption", () => {
  it("labels an office document and carries its size", () => {
    expect(spaceNodeCaption(file("Quarterly.docx", DOCX_BYTES))).toEqual({
      labelKey: "spaces.transfer.fileType.document",
      detail: "2.6 MiB",
    });
  });

  it("labels every preview kind a plain file can take", () => {
    expect(spaceNodeCaption(file("brief.pdf")).labelKey).toBe("spaces.transfer.fileType.pdf");
    expect(spaceNodeCaption(file("legacy.doc")).labelKey).toBe("spaces.transfer.fileType.document");
    expect(spaceNodeCaption(file("sheet.xlsx")).labelKey).toBe("spaces.transfer.fileType.spreadsheet");
    expect(spaceNodeCaption(file("deck.pptx")).labelKey).toBe("spaces.transfer.fileType.presentation");
    expect(spaceNodeCaption(file("notes.odt")).labelKey).toBe("spaces.transfer.fileType.file");
  });

  it("omits the detail rather than printing an empty one", () => {
    expect(spaceNodeCaption(file("notes.odt")).detail).toBeUndefined();
  });

  it("labels source code by what it is, not by its extension", () => {
    const code: SpaceCodePayload = {
      kind: "code",
      title: "main.rs",
      assetPath: "/spaces/assets/main.rs",
      fileName: "main.rs",
      previewLanguage: "rust",
      size: 2048,
    };
    expect(spaceNodeCaption(code)).toEqual({
      labelKey: "spaces.transfer.fileType.code",
      detail: "2.0 KiB",
    });
  });
});
