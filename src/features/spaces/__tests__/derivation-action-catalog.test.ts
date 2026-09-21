import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { creationIntentCatalog } from "../knowledge-derivations/catalog/creation-intent-catalog";
import { creationIntentDefinitions } from "../knowledge-derivations/catalog/creation-intent-definitions";

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
      title: `Reference ${index}`,
      fileName: `${index}.png`,
      assetPath: `/spaces/assets/${index}.png`,
      sourceRevision: `image:${index}`,
      visualKnowledge: {
        sourceRevision: `image:${index}`,
        description: `Reference image ${index}`,
        tags: [],
        generatedAt: 1,
        provider: "test",
        model: "vision",
      },
    },
    { x: index * 20, y: 0 },
    `image-${index}`,
  );

const indexedVideo = () =>
  createSpaceNode(
    {
      kind: "video",
      title: "Interview",
      fileName: "interview.mp4",
      assetPath: "/spaces/assets/interview.mp4",
      sourceRevision: "video:current",
      mediaKnowledge: {
        schemaVersion: 4,
        sourceRevision: "video:current",
        transcript: {
          language: "en",
          durationMs: 1_000,
          text: "Hello",
          segments: [{ startMs: 0, endMs: 1_000, text: "Hello" }],
          subtitleCues: [{ startMs: 0, endMs: 1_000, text: "Hello" }],
        },
      },
    },
    { x: 0, y: 0 },
    "video",
  );

describe("selection derivation catalog", () => {
  it("defines every selection action once with an explicit capability kind", () => {
    const ids = creationIntentDefinitions.map(({ id }) => id);

    expect(new Set(ids).size).toBe(ids.length);
    expect(creationIntentDefinitions.filter(({ kind }) => kind === "workflow").map(({ id }) => id)).toEqual([
      "presentation",
      "knowledge-video",
      "social-content-kit",
      "video-with-subtitles",
      "meeting-video-package",
    ]);
    expect(Object.keys(creationIntentCatalog([]).decisions)).toEqual(ids);
  });

  it("offers document, presentation and video generation from knowledge without requiring Markdown", () => {
    const decisions = creationIntentCatalog([note()]).decisions;

    expect(decisions["grounded-document"].available).toBe(true);
    expect(decisions["pdf-export"].available).toBe(false);
    expect(decisions.presentation.available).toBe(true);
    expect(decisions["knowledge-video"].available).toBe(true);
    expect(decisions["social-content-kit"].available).toBe(true);
  });

  it("rejects opaque resources instead of treating every selected file as knowledge", () => {
    const file = createSpaceNode(
      { kind: "file", title: "Binary", fileName: "binary.bin", assetPath: "/spaces/assets/binary.bin" },
      { x: 0, y: 0 },
      "file",
    );

    expect(creationIntentCatalog([file]).decisions["grounded-document"]).toEqual({
      available: false,
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

    expect(creationIntentCatalog([markdown]).decisions["docx-export"].available).toBe(true);
    expect(creationIntentCatalog([note()]).decisions["docx-export"].available).toBe(false);
  });

  it("keeps the provider reference-image limit explicit", () => {
    const nodes = [note(), ...Array.from({ length: 11 }, (_, index) => image(index))];

    expect(creationIntentCatalog(nodes).decisions["knowledge-video"]).toEqual({
      available: false,
      reason: "too-many-video-images",
    });
  });

  it("offers the complete subtitle Workflow for a current indexed video", () => {
    const decisions = creationIntentCatalog([indexedVideo()]).decisions;

    expect(decisions["video-with-subtitles"].available).toBe(true);
    expect(decisions["meeting-video-package"].available).toBe(true);
  });
});
