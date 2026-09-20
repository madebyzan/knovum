import { describe, expect, it } from "vitest";

import { registerSpaceSessionParticipant } from "../../canvas/editor-session";
import { createSpaceHistoryState } from "../../history/history-state";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import type { SpaceSessionContext } from "../space-session-context";
import { transitionActiveSpaceSession } from "../space-transition";

describe("space transition", () => {
  it("drains active mutations before the final canvas persistence flush", async () => {
    const calls: string[] = [];
    const { history } = createSpaceHistoryState();
    const releaseMutation = beginSpaceMutation(history);
    const detachContent = registerSpaceSessionParticipant({
      commit: async () => void calls.push("content"),
    });
    const detachPersistence = registerSpaceSessionParticipant({
      commit: async () => void calls.push("persistence"),
      phase: "persistence",
    });
    const detachLayout = registerSpaceSessionParticipant({
      commit: async () => void calls.push("layout"),
      phase: "layout",
    });
    const transition = transitionActiveSpaceSession(
      { history } as SpaceSessionContext,
      async () => void calls.push("transition"),
    );

    await Promise.resolve();
    await Promise.resolve();
    expect(calls).toEqual(["content"]);
    releaseMutation();
    await transition;

    expect(calls).toEqual(["content", "layout", "persistence", "transition"]);
    detachContent();
    detachPersistence();
    detachLayout();
  });
});
