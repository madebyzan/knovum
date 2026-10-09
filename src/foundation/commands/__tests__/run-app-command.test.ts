import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { isApplicationTransitioning, runApplicationTransition } from "../../lifecycle/application-transition";
import { runAppCommand } from "../run-app-command";

const mocks = vi.hoisted(() => ({
  logError: vi.fn<(message: string) => void>(),
  toastError: vi.fn<(message: string) => void>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));

function command(): AppCommand {
  return { id: "test.edit", title: "Edit", run: vi.fn<() => void>() };
}

beforeEach(() => {
  document.body.replaceChildren();
});

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

  it("blocks commands outside the active modal scope", async () => {
    const modal = document.createElement("div");
    modal.dataset.appCommandScope = "modal.confirm";
    modal.dataset.state = "open";
    document.body.append(modal);
    const edit = command();

    runAppCommand(edit);
    await Promise.resolve();

    expect(edit.run).not.toHaveBeenCalled();
  });

  it("allows a command declared by the active modal scope", async () => {
    const modal = document.createElement("div");
    modal.dataset.appCommandScope = "modal.confirm";
    modal.dataset.state = "open";
    document.body.append(modal);
    const confirm = { ...command(), id: "modal.confirm" };

    runAppCommand(confirm);
    await Promise.resolve();

    expect(confirm.run).toHaveBeenCalledOnce();
  });

  it("allows editing commands for an editable field inside the active modal", async () => {
    const modal = document.createElement("div");
    modal.dataset.appCommandScope = "";
    modal.dataset.state = "open";
    const input = document.createElement("input");
    modal.append(input);
    document.body.append(modal);
    input.focus();
    const copy = { ...command(), id: "editing.copy" };

    runAppCommand(copy);
    await Promise.resolve();

    expect(copy.run).toHaveBeenCalledOnce();
  });

  it("does not report an expected leave cancellation as a command failure", async () => {
    const cancelled = {
      ...command(),
      run: vi.fn<() => Promise<void>>(async () => {
        throw new ApplicationLeaveCancelled();
      }),
    };

    runAppCommand(cancelled);
    await vi.waitFor(() => expect(cancelled.run).toHaveBeenCalledOnce());

    expect(mocks.logError).not.toHaveBeenCalled();
    expect(mocks.toastError).not.toHaveBeenCalled();
  });
});
