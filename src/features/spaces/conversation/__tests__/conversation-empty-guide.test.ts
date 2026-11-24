import { describe, expect, it } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import {
  formatRecentConversationTimestamp,
  selectQuickContextItems,
  summarizeSpaceSources,
} from "../conversation-empty-guide";
import { buildEmptyConversationStarters } from "../conversation-empty-starters";

function makeNode(id: string, kind: SpaceCanvasNode["data"]["kind"]): SpaceCanvasNode {
  return {
    id,
    type: kind,
    data: { kind, title: id },
    initialPosition: { x: 0, y: 0 },
  } as SpaceCanvasNode;
}

describe("conversation empty guide", () => {
  it("summarizes semantic nodes by category and excludes non-semantic nodes", () => {
    const inventory = summarizeSpaceSources([
      makeNode("n1", "note"),
      makeNode("n2", "markdown"),
      makeNode("n3", "file"),
      makeNode("n4", "website"),
      makeNode("n5", "image"),
      makeNode("n6", "text"),
      makeNode("n7", "portal"),
    ]);

    expect(inventory.totalCount).toBe(5);
    expect(inventory.categories).toEqual([
      { category: "notes", count: 2 },
      { category: "documents", count: 1 },
      { category: "webpages", count: 1 },
    ]);
  });

  it("adapts starters for empty, single-source, multi-source, and active canvas selection", () => {
    expect(buildEmptyConversationStarters({ sourceCount: 0, selectedTitles: [] }).map((s) => s.id)).toEqual([
      "draft",
      "plan",
      "questions",
      "decision",
    ]);

    expect(buildEmptyConversationStarters({ sourceCount: 1, selectedTitles: [] }).map((s) => s.id)).toEqual([
      "overview",
      "brief",
      "takeaways",
      "gaps",
      "actions",
    ]);

    expect(buildEmptyConversationStarters({ sourceCount: 3, selectedTitles: [] }).map((s) => s.id)).toEqual([
      "overview",
      "brief",
      "compare",
      "gaps",
      "actions",
    ]);

    const withOneSelected = buildEmptyConversationStarters({
      sourceCount: 2,
      selectedTitles: ["Master Prompt"],
      maxCount: 4,
    });
    expect(withOneSelected.map((s) => s.id)).toEqual(["selection-single", "overview", "brief", "compare"]);
    expect(withOneSelected[0]?.attachSelection).toBe(true);
    expect(withOneSelected[0]?.params).toEqual({ title: "Master Prompt" });

    const withManySelected = buildEmptyConversationStarters({
      sourceCount: 3,
      selectedTitles: ["A", "B"],
      maxCount: 4,
    });
    expect(withManySelected[0]?.id).toBe("selection-multi");
    expect(withManySelected[0]?.params).toEqual({ count: 2 });
  });

  it("formats today, yesterday, and older recent conversation timestamps", () => {
    const now = new Date("2026-11-24T16:00:00");
    const today = new Date("2026-11-24T09:15:00").getTime();
    const yesterday = new Date("2026-11-23T18:00:00").getTime();
    const older = new Date("2026-11-10T10:00:00").getTime();

    expect(formatRecentConversationTimestamp(today, "en", "Yesterday", now)).toMatch(/09:15|9:15/);
    expect(formatRecentConversationTimestamp(yesterday, "en", "Yesterday", now)).toBe("Yesterday");
    expect(formatRecentConversationTimestamp(older, "en", "Yesterday", now)).toContain("10");
  });

  it("prioritizes selected and attached canvas sources in quick context items", () => {
    const nodes = [makeNode("n1", "note"), makeNode("n2", "website"), makeNode("n3", "file")];
    const items = selectQuickContextItems(
      [
        { nodeId: "n1", title: "Note 1", previewSource: "", selected: false, attached: false },
        { nodeId: "n2", title: "Web 2", previewSource: "", selected: false, attached: true },
        { nodeId: "n3", title: "File 3", previewSource: "", selected: true, attached: false },
      ],
      nodes,
      2,
    );
    expect(items.map((item) => [item.nodeId, item.nodeKind])).toEqual([
      ["n3", "file"],
      ["n2", "website"],
    ]);
  });
});
