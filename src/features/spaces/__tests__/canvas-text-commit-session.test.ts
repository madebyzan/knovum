import { describe, expect, it, vi } from "vitest";

import { flushSpaceSessionCommits, registerSpaceSessionParticipant } from "../canvas/editor-session";
import { deferred } from "./support/canvas-fixtures";

describe("canvas text commit session", () => {
  it("waits for every registered editor commit", async () => {
    const firstSave = deferred<void>();
    const first = vi.fn<() => Promise<void>>(() => firstSave.promise);
    const second = vi.fn<() => Promise<void>>(async () => {});
    const detachFirst = registerSpaceSessionParticipant({ commit: first });
    const detachSecond = registerSpaceSessionParticipant({ commit: second });

    const flush = flushSpaceSessionCommits();
    expect(first).toHaveBeenCalledOnce();
    expect(second).toHaveBeenCalledOnce();
    let complete = false;
    void flush.then(() => (complete = true));
    await Promise.resolve();
    expect(complete).toBe(false);

    firstSave.resolve();
    await flush;
    expect(complete).toBe(true);
    detachFirst();
    detachSecond();
  });

  it("commits content before persistence", async () => {
    const calls: string[] = [];
    const detachPersistence = registerSpaceSessionParticipant({
      commit: async () => void calls.push("persistence"),
      phase: "persistence",
    });
    const detachContent = registerSpaceSessionParticipant({
      commit: async () => void calls.push("content"),
    });

    await flushSpaceSessionCommits();

    expect(calls).toEqual(["content", "persistence"]);
    detachPersistence();
    detachContent();
  });
});
