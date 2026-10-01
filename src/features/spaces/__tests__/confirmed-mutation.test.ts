import { beforeEach, expect, it, vi } from "vitest";

const native = vi.hoisted(() => ({ invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
import { decodeAppError } from "../../../foundation/errors/app-error";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";

beforeEach(() => {
  native.invoke.mockReset();
});

it.each(["mutation_id_conflict", "operation_failed"])(
  "does not replace a returned %s failure with an older mutation receipt",
  async (code) => {
    native.invoke.mockRejectedValueOnce({ code, message: "Command rejected" });
    native.invoke.mockResolvedValueOnce({ mutationId: "used-id", appliedAtRevision: 1 });
    const mutationFailure = await invokeConfirmedMutation(
      "plugin:spaces|execute_space_edit",
      { request: { mutationId: "used-id" } },
      "used-id",
    ).catch((error: unknown) => error);
    expect(decodeAppError(mutationFailure)?.code).toBe(code);
    expect(native.invoke).toHaveBeenCalledTimes(1);
  },
);

it("still recovers a committed receipt after an ambiguous transport failure", async () => {
  const receipt = { mutationId: "new-id", appliedAtRevision: 1 };
  native.invoke.mockRejectedValueOnce(new Error("IPC connection closed"));
  native.invoke.mockResolvedValueOnce(receipt);
  await expect(
    invokeConfirmedMutation(
      "plugin:spaces|execute_space_edit",
      { request: { mutationId: "new-id" } },
      "new-id",
    ),
  ).resolves.toEqual(receipt);
  expect(native.invoke).toHaveBeenLastCalledWith("plugin:spaces|get_space_edit_receipt", {
    mutationId: "new-id",
  });
});

it("does not accept an old receipt when Tauri rejects arguments before entering the command", async () => {
  native.invoke.mockRejectedValueOnce("Invalid command arguments");
  native.invoke.mockResolvedValueOnce({ mutationId: "used-id", appliedAtRevision: 1 });
  const mutationFailure = await invokeConfirmedMutation(
    "plugin:spaces|execute_space_edit",
    { request: { mutationId: "used-id" } },
    "used-id",
  ).catch((error: unknown) => error);
  expect(mutationFailure).toBeInstanceOf(Error);
  expect(native.invoke).toHaveBeenCalledTimes(1);
});
