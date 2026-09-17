import type { z } from "zod";

export function parseStructuredJson<T>(raw: string, schema: z.ZodType<T>): T {
  const text = raw.trim();
  let jsonValue: unknown;
  try {
    jsonValue = JSON.parse(text);
  } catch {
    const fenceMatch = text.match(/```(?:json)?\s*([\s\S]*?)\s*```/i);
    if (fenceMatch?.[1]) {
      try {
        jsonValue = JSON.parse(fenceMatch[1].trim());
      } catch {
        // Fallthrough to brace extraction
      }
    }
    if (jsonValue === undefined) {
      const firstBrace = text.indexOf("{");
      const lastBrace = text.lastIndexOf("}");
      if (firstBrace !== -1 && lastBrace > firstBrace) {
        jsonValue = JSON.parse(text.slice(firstBrace, lastBrace + 1));
      } else {
        const firstBracket = text.indexOf("[");
        const lastBracket = text.lastIndexOf("]");
        if (firstBracket !== -1 && lastBracket > firstBracket) {
          jsonValue = JSON.parse(text.slice(firstBracket, lastBracket + 1));
        } else {
          throw new Error("Structured output does not contain valid JSON");
        }
      }
    }
  }
  return schema.parse(jsonValue);
}
