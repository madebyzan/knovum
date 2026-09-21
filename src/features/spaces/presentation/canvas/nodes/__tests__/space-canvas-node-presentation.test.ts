import { describe, expect, it } from "vitest";

import { createSpaceNotePayload } from "../../../../editor/tiptap-content";
import { spaceCanvasNodePresentation } from "../space-canvas-node-presentation";

describe("Konva node presentation", () => {
  it("keeps type-specific content instead of flattening every node into one card", () => {
    const note = spaceCanvasNodePresentation(createSpaceNotePayload("Note", "Body"));
    const website = spaceCanvasNodePresentation({
      kind: "website",
      title: "Knovum",
      url: "https://example.com",
      siteName: "Example",
    });

    expect(note).toMatchObject({ title: "Note", body: "Body", subtitle: "Note" });
    expect(website).toMatchObject({ title: "Knovum", subtitle: "Example" });
  });
});
