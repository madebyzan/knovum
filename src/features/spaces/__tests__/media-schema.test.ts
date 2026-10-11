import { describe, expect, it } from "vitest";

import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { transcriptFixture } from "./support/media-transcript-fixture";

const mediaKnowledge = {
  transcript: transcriptFixture("sha256:test", [
    { startMs: 0, endMs: 4000, text: "Hello" },
    { startMs: 4000, endMs: 10000, text: "world" },
  ]),
  sourceRevision: "sha256:test",
  schemaVersion: 1,
  insights: {
    summary: "Greeting",
    chapters: [{ startMs: 0, endMs: 10_000, title: "Greeting", summary: "A greeting" }],
    promptVersion: 1,
    generatedAt: 1,
    executionSource: { type: "model", provider: "provider", model: "model" },
  },
};

function audioWithKnowledge(knowledge: unknown) {
  return {
    kind: "audio",
    title: "Audio",
    assetPath: "/private/audio.wav",
    fileName: "audio.wav",
    sourceRevision: "sha256:test",
    mediaKnowledge: knowledge,
  };
}

describe("Space media knowledge schema", () => {
  it("rejects obsolete schema and foreign source revisions explicitly", () => {
    for (const schemaVersion of [0, 2, 4, 5]) {
      expect(() =>
        validateSpaceNodePayload(audioWithKnowledge({ ...mediaKnowledge, schemaVersion })),
      ).toThrow("schemaVersion");
    }
    const foreign = structuredClone(mediaKnowledge);
    foreign.transcript.sourceRevision = "sha256:another";
    expect(() => validateSpaceNodePayload(audioWithKnowledge(foreign))).toThrow(
      "Transcript source revision must match",
    );
  });
  it("rejects duplicate identities, partial timing and text ranges that misrepresent the source", () => {
    const duplicate = structuredClone(mediaKnowledge);
    duplicate.transcript.segments[1]!.id = duplicate.transcript.segments[0]!.id;
    expect(() => validateSpaceNodePayload(audioWithKnowledge(duplicate))).toThrow("normalized timing");
    const partial = structuredClone(mediaKnowledge);
    partial.transcript.segments[0]!.units[0]!.endMs = null;
    expect(() => validateSpaceNodePayload(audioWithKnowledge(partial))).toThrow("normalized timing");
    const mismatched = structuredClone(mediaKnowledge);
    mismatched.transcript.segments[0]!.units[0]!.text = "other";
    expect(() => validateSpaceNodePayload(audioWithKnowledge(mismatched))).toThrow("normalized timing");
    const shifted = structuredClone(mediaKnowledge);
    shifted.transcript.segments[0]!.endSample += 1;
    expect(() => validateSpaceNodePayload(audioWithKnowledge(shifted))).toThrow("normalized timing");
  });
  it("accepts transcript knowledge on audio and video items", () => {
    expect(
      validateSpaceNodePayload({
        kind: "video",
        title: "Clip",
        assetPath: "/private/clip.mp4",
        fileName: "clip.mp4",
        sourceRevision: "sha256:test",
        mediaKnowledge,
      }),
    ).toMatchObject({ kind: "video", mediaKnowledge });
  });

  it("accepts a local transcript before optional generative insights exist", () => {
    const { insights: _insights, ...localOnly } = structuredClone(mediaKnowledge);
    expect(
      validateSpaceNodePayload({
        kind: "audio",
        title: "Local transcript",
        assetPath: "/private/audio.wav",
        fileName: "audio.wav",
        sourceRevision: "sha256:test",
        mediaKnowledge: localOnly,
      }),
    ).toMatchObject({ kind: "audio", mediaKnowledge: localOnly });
  });

  it("does not allow media knowledge on unrelated node kinds", () => {
    expect(() =>
      validateSpaceNodePayload({
        kind: "file",
        title: "File",
        assetPath: "/private/file.pdf",
        fileName: "file.pdf",
        sourceRevision: "sha256:test",
        mediaKnowledge,
      }),
    ).toThrow("mediaKnowledge");
  });

  it("rejects transcript timestamps outside the media duration", () => {
    const invalid = structuredClone(mediaKnowledge);
    invalid.transcript.segments[1]!.endMs = 12_000;
    expect(() =>
      validateSpaceNodePayload({
        kind: "audio",
        title: "Audio",
        assetPath: "/private/audio.wav",
        fileName: "audio.wav",
        sourceRevision: "sha256:test",
        mediaKnowledge: invalid,
      }),
    ).toThrow("Transcript segments contain invalid normalized timing");
  });
  it("rejects chapters that are not anchored to transcript segments", () => {
    const invalid = structuredClone(mediaKnowledge);
    invalid.insights!.chapters[0]!.startMs = 1_000;
    expect(() =>
      validateSpaceNodePayload({
        kind: "video",
        title: "Clip",
        assetPath: "/private/clip.mp4",
        fileName: "clip.mp4",
        sourceRevision: "sha256:test",
        mediaKnowledge: invalid,
      }),
    ).toThrow("Chapters must be ordered, contiguous, and anchored to transcript segments");
  });
});
