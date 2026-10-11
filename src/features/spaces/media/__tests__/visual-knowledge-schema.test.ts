import { describe, expect, it } from "vitest";

import { spaceVisualKnowledgeSchema } from "../schema/visual-knowledge";

const knowledge = {
  sourceRevision: "source",
  description: "Visible diagram",
  tags: [],
  generatedAt: 1,
  executionSource: { type: "agent", runtimeKind: "codex" },
};

describe("visual knowledge result contract", () => {
  it("accepts UTF-16 boundaries and absent visible text", () => {
    expect(spaceVisualKnowledgeSchema.safeParse(knowledge).success).toBe(true);
    expect(
      spaceVisualKnowledgeSchema.safeParse({
        ...knowledge,
        description: "😀".repeat(4_000),
        detectedText: "😀".repeat(8_000),
        tags: Array.from({ length: 32 }, () => "😀".repeat(40)),
      }).success,
    ).toBe(true);
  });

  it("rejects oversized and empty fields", () => {
    for (const invalid of [
      { description: " " },
      { description: "😀".repeat(4_001) },
      { description: `${" ".repeat(8_000)}x` },
      { detectedText: "😀".repeat(8_001) },
      { tags: Array.from({ length: 33 }, () => "tag") },
      { tags: [" "] },
      { tags: ["😀".repeat(41)] },
    ]) {
      expect(spaceVisualKnowledgeSchema.safeParse({ ...knowledge, ...invalid }).success).toBe(false);
    }
  });
});
