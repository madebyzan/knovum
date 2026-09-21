import { describe, expect, it } from "vitest";

import { highlightedSnippet, isLocatorCurrent, knowledgeLocationLabel } from "../knowledge/model/location";
import type { KnowledgeLocator } from "../knowledge/model/schema";

function locator(location: KnowledgeLocator["location"], sourceRevision = "sha256:source"): KnowledgeLocator {
  return { sourceRevision, location, quote: { exact: "evidence" } };
}

describe("Space knowledge locations", () => {
  it("formats precise source positions", () => {
    expect(knowledgeLocationLabel(locator({ kind: "media", startMs: 83_000, endMs: 90_000 }))).toContain(
      "1:23",
    );
    expect(knowledgeLocationLabel(locator({ kind: "pdf", page: 42 }))).toBe("PDF · p.42");
    expect(knowledgeLocationLabel(locator({ kind: "slides", slide: 17 }))).toBe("Slides · 17");
    expect(knowledgeLocationLabel(locator({ kind: "sheet", sheet: "Revenue", range: "D18:F24" }))).toBe(
      "Sheet · Revenue · D18:F24",
    );
  });

  it("checks source and editor revisions against the correct authority", () => {
    const data = {
      kind: "file",
      title: "Doc",
      assetPath: "/managed/doc.pdf",
      fileName: "doc.pdf",
      sourceRevision: "sha256:source",
      knowledgeRevision: "node1:editor",
    } as const;
    expect(isLocatorCurrent(locator({ kind: "pdf", page: 1 }), data)).toBe(true);
    expect(isLocatorCurrent(locator({ kind: "node" }, "node1:editor"), data)).toBe(true);
    expect(isLocatorCurrent(locator({ kind: "pdf", page: 1 }, "sha256:old"), data)).toBe(false);
  });

  it("preserves explicit FTS match segments for rendering", () => {
    expect(highlightedSnippet("before ‹match› after")).toEqual([
      { text: "before ", match: false },
      { text: "match", match: true },
      { text: " after", match: false },
    ]);
  });
});
