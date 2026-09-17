import { describe, expect, it } from "vitest";

import { MAX_KNOWLEDGE_PASSAGE_BYTES, splitKnowledgePassages } from "../knowledge/model/passage";

describe("Knowledge passage splitting", () => {
  it("keeps every passage within the UTF-8 byte contract without splitting Unicode scalars", () => {
    const source = `heading\n${"😀alpha ".repeat(20_000)}`;
    const passages = splitKnowledgePassages(source);
    expect(passages.length).toBeGreaterThan(1);
    for (const passage of passages) {
      expect(new TextEncoder().encode(passage.text).byteLength).toBeLessThanOrEqual(
        MAX_KNOWLEDGE_PASSAGE_BYTES,
      );
      expect(passage.text.includes("\uFFFD")).toBe(false);
      expect(source.slice(passage.startOffset, passage.endOffset)).toBe(passage.text);
    }
  });

  it("drops whitespace-only input instead of creating an empty durable passage", () => {
    expect(splitKnowledgePassages("  \n\t  ")).toEqual([]);
  });
});
