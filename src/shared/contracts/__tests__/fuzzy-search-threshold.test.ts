import { describe, expect, it } from "vitest";

import { fuzzySearchThreshold } from "../fuzzy-search-threshold";

describe("fuzzySearchThreshold", () => {
  it("keeps the requested threshold for Latin queries", () => {
    expect(fuzzySearchThreshold("theme", 0.7)).toBe(0.7);
    expect(fuzzySearchThreshold("shortcuts")).toBe(0.6);
  });

  it("relaxes the threshold for CJK queries so short ideograph queries match reliably", () => {
    expect(fuzzySearchThreshold("主题", 0.7)).toBe(0.45);
    expect(fuzzySearchThreshold("ショートカット", 0.6)).toBe(0.45);
    expect(fuzzySearchThreshold("단축키", 0.6)).toBe(0.45);
  });
});
