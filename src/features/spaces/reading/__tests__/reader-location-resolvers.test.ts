import { describe, expect, it } from "vitest";

import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import {
  extractReaderRangeText,
  resolveChunkRangeLocation,
  resolveMediaRangeLocation,
  resolvePdfRangeLocation,
  resolveProseRangeLocation,
  resolveWebsiteRangeLocation,
} from "../reader-location-resolvers";

describe("reader-location-resolvers", () => {
  it("resolves PDF page number from the enclosing data-page-number container", () => {
    const host = document.createElement("div");
    host.innerHTML = '<div class="page" data-page-number="18"><span>Selected PDF passage</span></div>';
    const textNode = host.querySelector("span")!.firstChild!;
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 8);
    expect(resolvePdfRangeLocation(range, 1)).toEqual({ kind: "pdf", page: 18 });
    expect(extractReaderRangeText(range)).toBe("Selected");
  });

  it("resolves Website section heading and character offsets", () => {
    const host = document.createElement("div");
    host.innerHTML =
      '<article data-website-section="Research Findings"><p>The proposed approach improves retrieval quality.</p></article>';
    const textNode = host.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(textNode, 4);
    range.setEnd(textNode, 21);
    expect(resolveWebsiteRangeLocation(range)).toEqual({
      kind: "website",
      section: "Research Findings",
      startOffset: 4,
      endOffset: 21,
    });
  });

  it("resolves Media startMs and endMs across single or multiple transcript segments", () => {
    const host = document.createElement("div");
    host.innerHTML =
      '<div data-segment-start-ms="12000" data-segment-end-ms="15400"><p>First segment line.</p></div>' +
      '<div data-segment-start-ms="15400" data-segment-end-ms="19800"><p>Second segment line.</p></div>';
    const firstText = host.querySelectorAll("p")[0]!.firstChild!;
    const secondText = host.querySelectorAll("p")[1]!.firstChild!;
    const range = document.createRange();
    range.setStart(firstText, 0);
    range.setEnd(secondText, 6);
    expect(resolveMediaRangeLocation(range, host)).toEqual({
      kind: "media",
      startMs: 12_000,
      endMs: 19_800,
    });
  });

  it("resolves structured Document or Code chunk location by data-chunk-index", () => {
    const chunks: KnowledgeChunk[] = [
      {
        id: "chunk-0",
        spaceId: "space",
        nodeId: "doc",
        origin: "source",
        sourceRevision: "sha256:ppt",
        kind: "text",
        text: "Slide 3 summary",
        createdAt: 1,
        updatedAt: 1,
        locator: {
          sourceRevision: "sha256:ppt",
          location: { kind: "slides", slide: 3, element: 1 },
        },
      },
    ];
    const host = document.createElement("div");
    host.innerHTML = '<article data-chunk-index="0"><p>Slide 3 summary</p></article>';
    const textNode = host.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.selectNodeContents(textNode);
    expect(resolveChunkRangeLocation(range, chunks, { kind: "node" })).toEqual({
      kind: "slides",
      slide: 3,
      element: 1,
    });
  });

  it("resolves Markdown heading and offsets plus Note offsets inside reader-prose containers", () => {
    const host = document.createElement("div");
    host.innerHTML =
      "<div data-reader-prose><h2>Retrieval Quality</h2><p>Hybrid indexing reduces latency.</p></div>";
    const textNode = host.querySelector("p")!.firstChild!;
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 15);
    expect(resolveProseRangeLocation(range, "markdown")).toMatchObject({
      kind: "markdown",
      heading: "Retrieval Quality",
    });
    expect(resolveProseRangeLocation(range, "note")).toMatchObject({
      kind: "note",
      startOffset: expect.any(Number),
      endOffset: expect.any(Number),
    });
  });
});
