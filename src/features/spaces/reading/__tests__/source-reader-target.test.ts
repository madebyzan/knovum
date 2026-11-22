import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../../canvas/node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import {
  buildSourceReaderLocator,
  classifyReadableSource,
  defaultSourceLocation,
  isReadableSourceNode,
} from "../source-reader-target";

function makeNode(id: string, data: SpaceNodePayload) {
  return createSpaceNode(data, { x: 0, y: 0 }, id);
}

describe("source-reader-target", () => {
  it("classifies PDF, Word, Website, Media, Code, Markdown, and Note nodes as readable sources", () => {
    const pdf = makeNode("pdf-1", {
      kind: "file",
      title: "Paper",
      fileName: "paper.pdf",
      assetPath: "/assets/paper.pdf",
      sourceRevision: "sha256:pdf1",
    });
    const docx = makeNode("doc-1", {
      kind: "file",
      title: "Report",
      fileName: "report.docx",
      assetPath: "/assets/report.docx",
      sourceRevision: "sha256:doc1",
    });
    const pptx = makeNode("ppt-1", {
      kind: "file",
      title: "Deck",
      fileName: "deck.pptx",
      assetPath: "/assets/deck.pptx",
      sourceRevision: "sha256:ppt1",
    });
    const archive = makeNode("zip-1", {
      kind: "file",
      title: "Archive",
      fileName: "bundle.zip",
      assetPath: "/assets/bundle.zip",
      entries: [{ path: "a.txt", kind: "file" }],
    });
    const website = makeNode("web-1", {
      kind: "website",
      title: "Article",
      url: "https://example.com/post",
      knowledgeRevision: "node1:web1",
    });
    const video = makeNode("vid-1", {
      kind: "video",
      title: "Lecture",
      fileName: "lecture.mp4",
      assetPath: "/assets/lecture.mp4",
      sourceRevision: "sha256:vid1",
      durationSeconds: 120,
    });
    const code = makeNode("code-1", {
      kind: "code",
      title: "main.rs",
      fileName: "main.rs",
      assetPath: "/assets/main.rs",
      previewLanguage: "rust",
      sourceRevision: "sha256:code1",
    });
    const markdown = makeNode("md-1", {
      kind: "markdown",
      title: "Architecture Notes",
      text: "# Intro\nBody",
    });
    const note = makeNode("note-1", {
      kind: "note",
      title: "Synthesis",
      plainText: "Summary",
      tiptapContent: {
        type: "doc",
        content: [{ type: "paragraph", content: [{ type: "text", text: "Summary" }] }],
      },
    });

    const csv = makeNode("csv-1", {
      kind: "file",
      title: "Metrics",
      fileName: "metrics.csv",
      assetPath: "/assets/metrics.csv",
      sourceRevision: "sha256:csv1",
    });
    const image = makeNode("img-1", {
      kind: "image",
      title: "Architecture Diagram",
      fileName: "arch.png",
      assetPath: "/assets/arch.png",
      sourceRevision: "sha256:img1",
    });

    expect(classifyReadableSource(pdf.data)).toBe("pdf");
    expect(classifyReadableSource(docx.data)).toBe("word");
    expect(classifyReadableSource(pptx.data)).toBe("slides");
    expect(classifyReadableSource(csv.data)).toBe("document");
    expect(classifyReadableSource(image.data)).toBe("image");
    expect(classifyReadableSource(archive.data)).toBeUndefined();
    expect(classifyReadableSource(website.data)).toBe("website");
    expect(classifyReadableSource(video.data)).toBe("media");
    expect(classifyReadableSource(code.data)).toBe("code");
    expect(classifyReadableSource(markdown.data)).toBe("markdown");
    expect(classifyReadableSource(note.data)).toBe("note");
    expect(isReadableSourceNode(pdf)).toBe(true);
    expect(isReadableSourceNode(archive)).toBe(false);
    expect(defaultSourceLocation(docx.data)).toEqual({
      kind: "word",
      section: 0,
      element: 0,
      headingPath: [],
    });
    expect(defaultSourceLocation(markdown.data)).toEqual({ kind: "markdown" });
    expect(defaultSourceLocation(note.data)).toEqual({ kind: "note" });
  });

  it("builds locators preserving the source revision, location, and exact quote", () => {
    const payload: SpaceNodePayload = {
      kind: "file",
      title: "Paper",
      fileName: "paper.pdf",
      assetPath: "/assets/paper.pdf",
      sourceRevision: "sha256:rev18",
    };
    const locator = buildSourceReaderLocator(
      payload,
      { kind: "pdf", page: 18 },
      "Retrieval quality improves downstream accuracy.",
    );
    expect(locator).toEqual({
      sourceRevision: "sha256:rev18",
      location: { kind: "pdf", page: 18 },
      quote: { exact: "Retrieval quality improves downstream accuracy." },
    });
    expect(defaultSourceLocation(payload, 12)).toEqual({ kind: "pdf", page: 12 });
  });
});
