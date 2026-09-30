import { describe, expect, it } from "vitest";
import { reactive } from "vue";

import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";

describe("createSpaceNode", () => {
  it("keeps renderer state out of the domain node contract", () => {
    const data = createSpaceNotePayload("Reference", "Body");
    const node = createSpaceNode(data, { x: 40, y: 80 }, "note-1");

    expect(node).toEqual({
      id: "note-1",
      type: "note",
      initialPosition: { x: 40, y: 80 },
      data,
    });
  });

  it("rejects Structure fields in content data", () => {
    expect(() =>
      validateSpaceNodePayload({ ...createSpaceNotePayload("Reference"), sectionId: "section" }),
    ).toThrow(/unrecognized_key/u);
  });

  it("deep-clones media knowledge for independent duplicated nodes", () => {
    const source = createSpaceNode(
      {
        kind: "audio",
        title: "Interview",
        assetPath: "/spaces/assets/interview.wav",
        fileName: "interview.wav",
        mediaKnowledge: {
          transcript: {
            language: "en",
            durationMs: 1_000,
            text: "Hello",
            segments: [{ startMs: 0, endMs: 1_000, text: "Hello" }],
            subtitleCues: [{ startMs: 0, endMs: 1_000, text: "Hello" }],
          },
          sourceRevision: "sha256:test",
          schemaVersion: 4,
          insights: {
            summary: "Greeting",
            chapters: [{ startMs: 0, endMs: 1_000, title: "Intro", summary: "Greeting" }],
            promptVersion: 1,
            generatedAt: 1,
            executionSource: { type: "model", provider: "provider", model: "model" },
          },
        },
      },
      { x: 0, y: 0 },
      "audio-1",
    );

    const cloned = createSpaceNode(validateSpaceNodePayload(source.data), { x: 20, y: 20 }, "audio-2");
    if (source.data.kind !== "audio" || cloned.data.kind !== "audio") throw new Error("Expected audio nodes");
    cloned.data.mediaKnowledge!.transcript.segments[0]!.text = "Changed";
    cloned.data.mediaKnowledge!.insights!.chapters[0]!.title = "Changed";

    expect(source.data.mediaKnowledge!.transcript.segments[0]!.text).toBe("Hello");
    expect(source.data.mediaKnowledge!.insights!.chapters[0]!.title).toBe("Intro");
  });

  it("clones reactive node data into a plain persistable value", () => {
    const source = reactive(createSpaceNotePayload("Idea", "Body"));
    const clone = validateSpaceNodePayload(source);

    expect(clone).toEqual(source);
    expect(() => structuredClone(clone)).not.toThrow();
  });
});
