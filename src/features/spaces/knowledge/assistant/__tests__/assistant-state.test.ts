import { describe, expect, it } from "vitest";

import {
  activeThreadNodeIds,
  knowledgeScopeNodeIds,
  suggestedKnowledgeScope,
} from "../assistant-state";
import type { GroundedThread } from "../../model/grounded-thread";

describe("grounded assistant scope", () => {
  it("keeps persisted thread scope authoritative", () => {
    const thread: GroundedThread = {
      id: "thread",
      spaceId: "space",
      scope: "selection",
      scopeNodeIds: ["a", "b"],
      title: "Compare",
      createdAt: 1,
      updatedAt: 1,
    };
    expect(activeThreadNodeIds(thread, ["current-selection"])).toEqual(["a", "b"]);
    expect(knowledgeScopeNodeIds("section", ["a"], ["b", "c"])).toEqual(["b", "c"]);
  });

  it("suggests the narrowest useful scope for a new thread", () => {
    expect(suggestedKnowledgeScope(["selected"], ["section"])).toBe("selection");
    expect(suggestedKnowledgeScope([], ["section"])).toBe("section");
    expect(suggestedKnowledgeScope([], [])).toBe("space");
  });
});
