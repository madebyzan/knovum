import { describe, expect, it } from "vitest";

import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

const mediaKnowledge = {
  transcript: {
    language: "en",
    durationMs: 10_000,
    text: "Hello world",
    segments: [
      { startMs: 0, endMs: 4_000, text: "Hello" },
      { startMs: 4_000, endMs: 10_000, text: "world" },
    ],
    subtitleCues: [
      { startMs: 0, endMs: 4_000, text: "Hello" },
      { startMs: 4_000, endMs: 10_000, text: "world" },
    ],
  },
  sourceRevision: "sha256:test",
  schemaVersion: 4,
  insights: {
    summary: "Greeting",
    chapters: [{ startMs: 0, endMs: 10_000, title: "Greeting", summary: "A greeting" }],
    promptVersion: 1,
    generatedAt: 1,
    provider: "provider",
    model: "model",
  },
};

describe("Space media knowledge schema", () => {
  it("accepts transcript knowledge on audio and video items", () => {
    expect(
      validateSpaceNodePayload({
        kind: "video",
        title: "Clip",
        assetPath: "/private/clip.mp4",
        fileName: "clip.mp4",
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
        mediaKnowledge: invalid,
      }),
    ).toThrow("Transcript segments must be ordered and bounded");
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
        mediaKnowledge: invalid,
      }),
    ).toThrow("Chapters must be ordered, contiguous, and anchored to transcript segments");
  });
});
