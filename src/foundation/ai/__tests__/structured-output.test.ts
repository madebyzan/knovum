import { describe, expect, it } from "vitest";
import { z } from "zod";

import { parseStructuredJson } from "../structured-output";

const sampleSchema = z.object({
  title: z.string(),
  tags: z.array(z.string()).default([]),
});

describe("parseStructuredJson", () => {
  it("parses clean JSON string", () => {
    const input = JSON.stringify({ title: "Test Title", tags: ["demo", "test"] });
    const parsed = parseStructuredJson(input, sampleSchema);
    expect(parsed).toEqual({ title: "Test Title", tags: ["demo", "test"] });
  });

  it("extracts and parses markdown code-fenced JSON", () => {
    const input = '```json\n{\n  "title": "Fenced Title",\n  "tags": ["demo"]\n}\n```';
    const parsed = parseStructuredJson(input, sampleSchema);
    expect(parsed).toEqual({ title: "Fenced Title", tags: ["demo"] });
  });

  it("extracts and parses JSON wrapped in generic code fences", () => {
    const input = '```\n{\n  "title": "Generic Fence",\n  "tags": []\n}\n```';
    const parsed = parseStructuredJson(input, sampleSchema);
    expect(parsed).toEqual({ title: "Generic Fence", tags: [] });
  });

  it("extracts and parses JSON with conversational preamble and epilogue", () => {
    const input =
      'Here is the requested analysis:\n```json\n{"title": "With Intro", "tags": ["nlp"]}\n```\nHope this helps!';
    const parsed = parseStructuredJson(input, sampleSchema);
    expect(parsed).toEqual({ title: "With Intro", tags: ["nlp"] });
  });

  it("throws when JSON does not match schema", () => {
    const input = JSON.stringify({ wrongField: 123 });
    expect(() => parseStructuredJson(input, sampleSchema)).toThrow("Invalid input");
  });
});
