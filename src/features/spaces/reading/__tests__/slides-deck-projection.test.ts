import { describe, expect, it, vi } from "vitest";

import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceFilePayload } from "../../model/nodes/node-payload";
import { projectPresentationSlides } from "../slides-deck-projection";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

const RES_1 = "a".repeat(64);
const RES_2 = "b".repeat(64);

describe("slides-deck-projection", () => {
  it("groups slide visuals, extracted chunks, and OCR passages while skipping node summary chunks", () => {
    const payload: SpaceFilePayload = {
      kind: "file",
      title: "Git Guide.pptx",
      fileName: "Git Guide.pptx",
      assetPath: "/assets/git.pptx",
      sourceRevision: "sha256:ppt1",
      documentCoverage: {
        schemaVersion: 1,
        parser: "office-oxide",
        sourceRevision: "sha256:ppt1",
        pageCount: 2,
        pagesRequiringRecognition: [],
        pagesWithExtractedText: [2],
        pagesWithoutExtractableContent: [],
        resources: [
          {
            id: RES_1,
            path: "/assets/slide-1.png",
            mediaType: "image/png",
            width: 1920,
            height: 1080,
            byteSize: 1024,
          },
          {
            id: RES_2,
            path: "/assets/slide-2.png",
            mediaType: "image/png",
            width: 1920,
            height: 1080,
            byteSize: 1024,
          },
        ],
        occurrences: [
          {
            id: "occ-1",
            location: { kind: "slides", slide: 1 },
            context: "",
            altText: null,
            resourceId: RES_1,
            pdfInput: null,
            unavailable: null,
          },
          {
            id: "occ-2",
            location: { kind: "slides", slide: 2 },
            context: "",
            altText: null,
            resourceId: RES_2,
            pdfInput: null,
            unavailable: null,
          },
        ],
        warnings: [],
      },
    };
    const chunks: KnowledgeChunk[] = [
      {
        id: "c0",
        spaceId: "space",
        nodeId: "ppt",
        origin: "source",
        sourceRevision: "sha256:ppt1",
        kind: "node",
        text: "Git Guide.pptx Git Guide.pptx",
        createdAt: 1,
        updatedAt: 1,
        locator: { sourceRevision: "sha256:ppt1", location: { kind: "node" } },
      },
      {
        id: "c1",
        spaceId: "space",
        nodeId: "ppt",
        origin: "source",
        sourceRevision: "sha256:ppt1",
        kind: "slide",
        text: "Branching rules",
        createdAt: 1,
        updatedAt: 1,
        locator: { sourceRevision: "sha256:ppt1", location: { kind: "slides", slide: 2, element: 0 } },
      },
    ];

    const slides = projectPresentationSlides(payload, chunks, (id) =>
      id === RES_1 ? "Recognized Slide 1 title" : undefined,
    );
    expect(slides).toHaveLength(2);
    expect(slides[0]!.slideNumber).toBe(1);
    expect(slides[0]!.images[0]!.src).toBe("asset://localhost//assets/slide-1.png");
    expect(slides[0]!.ocrPassages).toEqual(["Recognized Slide 1 title"]);
    expect(slides[0]!.textBlocks).toEqual([]);
    expect(slides[1]!.slideNumber).toBe(2);
    expect(slides[1]!.textBlocks).toEqual([{ chunkIndex: 1, text: "Branching rules" }]);
  });
});
