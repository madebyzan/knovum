import { describe, expect, it } from "vitest";

import { markSpaceSubstring, searchTerms } from "../search/search-snippet";

describe("Space search snippets", () => {
  it("derives snippet terms while Rust owns the database search plan", () => {
    expect(searchTerms("“数据库” 一致性")).toEqual(["数据库", "一致性"]);
    expect(searchTerms('foo OR "bar"')).toEqual(["foo", "OR", "bar"]);
  });

  it("marks the earliest literal substring match without treating search text as wildcards", () => {
    expect(markSpaceSubstring("alpha 100%_done omega", ["100%_"])).toContain("‹100%_›");
    expect(markSpaceSubstring("alpha beta", ["missing"])).toBe("alpha beta");
  });
});
