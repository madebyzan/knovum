import { describe, expect, it, vi } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { isApplicationTransitioning, runApplicationTransition } from "../../lifecycle/application-transition";
import { runAppCommand } from "../run-app-command";

vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => void>() }));
vi.mock("vue-sonner", () => ({ toast: { error: vi.fn<(message: string) => void>() } }));

function command(): AppCommand {
  return { id: "test.edit", title: "Edit", run: vi.fn<() => void>() };
}

describe("application commands during lifecycle transitions", () => {
  it("does not execute menu or shortcut commands while saving for installation", async () => {
    const edit = command();
    await runApplicationTransition(async () => {
      runAppCommand(edit);
      await Promise.resolve();
      expect(edit.run).not.toHaveBeenCalled();
    });
    runAppCommand(edit);
    await Promise.resolve();
    expect(edit.run).toHaveBeenCalledOnce();
  });

  it("rechecks commands queued before the transition acquired the lock", async () => {
    const edit = command();
    runAppCommand(edit);
    await runApplicationTransition(async () => {
      await Promise.resolve();
      expect(edit.run).not.toHaveBeenCalled();
    });
  });

  it("does not release an existing transition when a competing transition is rejected", async () => {
    await runApplicationTransition(async () => {
      const competing = vi.fn<() => Promise<void>>(async () => undefined);
      await expect(runApplicationTransition(competing)).rejects.toThrow("already running");
      expect(competing).not.toHaveBeenCalled();
      expect(isApplicationTransitioning()).toBe(true);
    });
    expect(isApplicationTransitioning()).toBe(false);
  });

  it("releases the admission lock after an operation fails", async () => {
    await expect(
      runApplicationTransition(async () => {
        throw new Error("Save failed");
      }),
    ).rejects.toThrow("Save failed");
    expect(isApplicationTransitioning()).toBe(false);
    const edit = command();
    runAppCommand(edit);
    await Promise.resolve();
    expect(edit.run).toHaveBeenCalledOnce();
  });
});
