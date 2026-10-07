import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { creationIntentCatalog } from "../knowledge-derivations/catalog/creation-intent-catalog";
import { creationIntentDefinitions } from "../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { transcriptFixture } from "./media-transcript-fixture";

const note = () =>
  createSpaceNode(
    {
      kind: "note",
      title: "Research note",
      plainText: "A grounded observation.",
      tiptapContent: { type: "doc", content: [] },
      knowledgeRevision: "note:current",
    },
    { x: 0, y: 0 },
    "note",
  );

const image = (index: number) =>
  createSpaceNode(
    {
      kind: "image",
      knowledgeRevision: "node1:0000000000000001",
      title: `Reference ${index}`,
      fileName: `${index}.png`,
      assetPath: `/spaces/assets/${index}.png`,
      sourceRevision: `image:${index}`,
      visualKnowledge: {
        sourceRevision: `image:${index}`,
        description: `Reference image ${index}`,
        tags: [],
        generatedAt: 1,
        executionSource: { type: "model", provider: "test", model: "vision" },
      },
    },
    { x: index * 20, y: 0 },
    `image-${index}`,
  );

const indexedVideo = () =>
  createSpaceNode<SpaceVideoPayload>(
    {
      kind: "video",
      knowledgeRevision: "node1:0000000000000002",
      title: "Interview",
      fileName: "interview.mp4",
      assetPath: "/spaces/assets/interview.mp4",
      sourceRevision: "video:current",
      mediaKnowledge: {
        schemaVersion: 1,
        sourceRevision: "video:current",
        transcript: transcriptFixture("video:current", [{ startMs: 0, endMs: 1000, text: "Hello" }]),
      },
    },
    { x: 0, y: 0 },
    "video",
  );

describe("selection derivation catalog", () => {
  it("defines each capability once without built-in workflow recipes", () => {
    const ids = creationIntentDefinitions.map(({ id }) => id);

    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).not.toContain("video-production");
    expect(ids).not.toContain("presentation-production");
    expect(ids).not.toContain("meeting-content-package");
    expect(ids).not.toContain("social-content-package");
    expect(Object.keys(creationIntentCatalog([]).decisions)).toEqual(ids);
  });

  it("offers document, presentation and video generation from knowledge without requiring Markdown", () => {
    const decisions = creationIntentCatalog([note()]).decisions;

    expect(decisions["draft-document"].state).toBe("ready");
    expect(decisions["export-document-as-pdf"].state).toBe("hidden");
    expect(decisions["render-presentation"].state).toBe("hidden");
    expect(decisions["generate-video"].state).toBe("ready");
  });

  it("rejects opaque resources instead of treating every selected file as knowledge", () => {
    const file = createSpaceNode(
      {
        kind: "file",
        title: "Binary",
        fileName: "binary.bin",
        assetPath: "/spaces/assets/binary.bin",
      },
      { x: 0, y: 0 },
      "file",
    );

    expect(creationIntentCatalog([file]).decisions["draft-document"]).toEqual({
      state: "blocked",
      reason: "needs-indexed-knowledge",
    });
  });

  it("exports only an explicit current Markdown resource instead of hiding a synthesis step", () => {
    const markdown = createSpaceNode(
      {
        kind: "markdown",
        title: "Brief",
        text: "# Brief",
        knowledgeRevision: "markdown:current",
      },
      { x: 0, y: 0 },
      "markdown",
    );

    expect(creationIntentCatalog([markdown]).decisions["export-document-as-docx"].state).toBe("ready");
    expect(creationIntentCatalog([note()]).decisions["export-document-as-docx"].state).toBe("hidden");
  });

  it("keeps the provider reference-image limit explicit", () => {
    const nodes = [note(), ...Array.from({ length: 11 }, (_, index) => image(index))];

    expect(creationIntentCatalog(nodes).decisions["generate-video"]).toEqual({
      state: "blocked",
      reason: "too-many-video-images",
    });
  });

  it("offers an atomic subtitle derivation for a current video", () => {
    const decisions = creationIntentCatalog([indexedVideo()]).decisions;

    expect(decisions["add-video-subtitles"].state).toBe("ready");
  });
});

it("adds subtitles without requiring published transcript knowledge", () => {
  const video = indexedVideo();
  if (video.data.kind !== "video") throw new Error("Video fixture expected");
  delete video.data.mediaKnowledge;
  const decisions = creationIntentCatalog([video]).decisions;
  expect(decisions["add-video-subtitles"].state).toBe("ready");
  expect(creationIntentDefinitions.map(({ id }) => id)).not.toContain("extract-subtitles");
  expect(creationIntentCatalog([video, note()]).decisions["add-video-subtitles"].state).toBe("hidden");
  video.data.processingState = "processing";
  expect(creationIntentCatalog([video]).decisions["add-video-subtitles"].state).toBe("hidden");
});
