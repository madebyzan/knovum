import { describe, expect, it } from "vitest";

import { createKeyedSerialQueue } from "../keyed-serial-queue";

describe("createKeyedSerialQueue", () => {
  it("keeps three writes for the same key in submission order", async () => {
    const queue = createKeyedSerialQueue<string>();
    const events: number[] = [];
    let releaseFirst!: () => void;
    const firstGate = new Promise<void>((resolve) => (releaseFirst = resolve));

    const first = queue.run("settings", async () => {
      await firstGate;
      events.push(1);
    });
    const second = queue.run("settings", async () => events.push(2));
    const third = queue.run("settings", async () => events.push(3));
    releaseFirst();
    await Promise.all([first, second, third]);

    expect(events).toEqual([1, 2, 3]);
  });

  it("does not block independent keys", async () => {
    const queue = createKeyedSerialQueue<string>();
    const events: string[] = [];
    let releaseA!: () => void;
    const gateA = new Promise<void>((resolve) => (releaseA = resolve));
    const a = queue.run("a", async () => {
      await gateA;
      events.push("a");
    });
    await queue.run("b", async () => events.push("b"));
    releaseA();
    await a;
    expect(events).toEqual(["b", "a"]);
  });

  it("preserves a failed write's error and continues queued writes for its key", async () => {
    const queue = createKeyedSerialQueue<string>();
    const cause = new Error("disk unavailable");
    const first = queue.run("node", async () => {
      throw cause;
    });
    const second = queue.run("node", async () => "saved");

    await expect(first).rejects.toBe(cause);
    await expect(second).resolves.toBe("saved");
    await expect(queue.run("node", async () => "saved again")).resolves.toBe("saved again");
  });
});
