import { describe, expect, it } from "vitest";

import { derivationSchema } from "../derivation-schema";

function generatedVideoDerivation() {
  return {
    id: "derivation",
    spaceId: "space",
    outputNodeId: "video",
    inputs: [{ nodeId: "markdown", role: "primary", sourceRevision: "node1:source" }],
    actionKind: "knowledge-video",
    actionVersion: 1,
    engine: "qwen:wan3.0-video",
    status: "completed",
    error: "",
    createdAt: 1,
    updatedAt: 2,
    completedAt: 2,
    generatedVideo: {
      prompt: "Explain the source knowledge visually",
      aspectRatio: "16:9",
      durationSeconds: 10,
      provider: "qwen",
      model: "wan3.0-video",
      operationId: "task-1",
    },
  } as const;
}

describe("knowledge video package metadata", () => {
  it("accepts complete generated video metadata", () => {
    expect(derivationSchema.safeParse(generatedVideoDerivation()).success).toBe(true);
  });

  it("requires metadata for a completed knowledge video", () => {
    const { generatedVideo: _generatedVideo, ...derivation } = generatedVideoDerivation();
    expect(derivationSchema.safeParse(derivation).success).toBe(false);
  });

  it("rejects obsolete review metadata", () => {
    const derivation = generatedVideoDerivation();
    const generatedVideo = { ...derivation.generatedVideo, reviewState: "reviewed" };
    expect(derivationSchema.safeParse({ ...derivation, generatedVideo }).success).toBe(false);
  });
});

describe("document export package metadata", () => {
  it("accepts a completed Markdown PDF export without provider metadata", () => {
    expect(
      derivationSchema.safeParse({
        id: "pdf-export",
        spaceId: "space",
        outputNodeId: "pdf",
        inputs: [{ nodeId: "markdown", role: "primary", sourceRevision: "node1:source" }],
        actionKind: "markdown-pdf-export",
        actionVersion: 1,
        engine: "typst:0.15.1",
        status: "completed",
        error: "",
        createdAt: 1,
        updatedAt: 2,
        completedAt: 2,
      }).success,
    ).toBe(true);
  });
});
