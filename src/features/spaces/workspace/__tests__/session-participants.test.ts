import { describe, expect, it, onTestFinished } from "vitest";

import {
  flushSpaceSessionCommits,
  registerSpaceSessionParticipant,
  type SpaceSessionParticipant,
} from "../session-participants";

function register(participant: SpaceSessionParticipant) {
  const detach = registerSpaceSessionParticipant(participant);
  onTestFinished(detach);
}

describe("Space session participants", () => {
  it("prepares every phase in order before finalizing any participant", async () => {
    const calls: string[] = [];
    register({
      phase: "persistence",
      commit: async () => void calls.push("prepare persistence"),
      finalize: () => void calls.push("finalize persistence"),
    });
    register({
      phase: "layout",
      commit: async () => void calls.push("prepare layout"),
      finalize: () => void calls.push("finalize layout"),
    });
    register({
      commit: async () => void calls.push("prepare content"),
      finalize: () => void calls.push("finalize content"),
    });

    await flushSpaceSessionCommits();

    expect(calls).toEqual([
      "prepare content",
      "prepare layout",
      "prepare persistence",
      "finalize persistence",
      "finalize layout",
      "finalize content",
    ]);
  });

  it.each(["layout", "persistence"] as const)(
    "does not finalize an accepted content discard when %s preparation fails",
    async (failedPhase) => {
      const failure = new Error(`${failedPhase} failed`);
      const calls: string[] = [];
      register({
        commit: async () => void calls.push("prepare content discard"),
        finalize: () => void calls.push("discard content"),
      });
      register({
        phase: "layout",
        commit: async () => {
          calls.push("prepare layout");
          if (failedPhase === "layout") throw failure;
        },
      });
      register({
        phase: "persistence",
        commit: async () => {
          calls.push("prepare persistence");
          if (failedPhase === "persistence") throw failure;
        },
      });

      await expect(flushSpaceSessionCommits()).rejects.toBe(failure);

      expect(calls).not.toContain("discard content");
      expect(calls.includes("prepare persistence")).toBe(failedPhase === "persistence");
    },
  );
});
