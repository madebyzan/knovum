import { describe, expect, it, onTestFinished, vi } from "vitest";

import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { flushSpaceSessionCommits, registerSpaceSessionParticipant } from "../workspace/session-participants";
import { deferred } from "./support/canvas-fixtures";

describe("canvas text commit session", () => {
  it("runs registered editor commits one at a time", async () => {
    const firstSave = deferred<void>();
    const first = vi.fn<() => Promise<void>>(() => firstSave.promise);
    const second = vi.fn<() => Promise<void>>(async () => {});
    const detachFirst = registerSpaceSessionParticipant({ commit: first });
    const detachSecond = registerSpaceSessionParticipant({ commit: second });
    onTestFinished(() => {
      detachFirst();
      detachSecond();
    });

    const flush = flushSpaceSessionCommits();
    expect(first).toHaveBeenCalledOnce();
    expect(second).not.toHaveBeenCalled();

    firstSave.resolve();
    await flush;

    expect(second).toHaveBeenCalledOnce();
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
    onTestFinished(() => {
      detachPersistence();
      detachContent();
    });

    await flushSpaceSessionCommits();

    expect(calls).toEqual(["content", "persistence"]);
  });

  it("rejects the flush without starting later commits after a save failure", async () => {
    const saveFailure = new Error("Save failed");
    const later = vi.fn<() => Promise<void>>(async () => {});
    const detachFailed = registerSpaceSessionParticipant({
      commit: vi.fn<() => Promise<void>>(async () => {
        throw saveFailure;
      }),
    });
    const detachLater = registerSpaceSessionParticipant({ commit: later });
    onTestFinished(() => {
      detachFailed();
      detachLater();
    });

    await expect(flushSpaceSessionCommits()).rejects.toBe(saveFailure);

    expect(later).not.toHaveBeenCalled();
  });

  it("stops later commits when an editor cancels leaving", async () => {
    const later = vi.fn<() => Promise<void>>(async () => {});
    const detachCancelled = registerSpaceSessionParticipant({
      commit: vi.fn<() => Promise<void>>(async () => {
        throw new ApplicationLeaveCancelled();
      }),
    });
    const detachLater = registerSpaceSessionParticipant({ commit: later });
    onTestFinished(() => {
      detachCancelled();
      detachLater();
    });

    await expect(flushSpaceSessionCommits()).rejects.toBeInstanceOf(ApplicationLeaveCancelled);

    expect(later).not.toHaveBeenCalled();
  });
});
