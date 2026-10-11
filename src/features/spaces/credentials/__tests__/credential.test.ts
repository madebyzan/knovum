import { beforeEach, describe, expect, it, vi } from "vitest";

const secrets = vi.hoisted(() => ({
  deleteSecret: vi.fn<(key: string) => Promise<void>>(),
  getSecret: vi.fn<(key: string) => Promise<string | null>>(),
  setSecret: vi.fn<(key: string, value: string) => Promise<void>>(),
}));

vi.mock("../../../../foundation/secrets/secret-store", () => secrets);

import { commitSpaceCredentialChange } from "../credential-actions";
import { deleteSpaceCredentials } from "../credential-storage";

describe("spaces credential consistency", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    secrets.deleteSecret.mockResolvedValue();
    secrets.setSecret.mockResolvedValue();
  });

  it("stops bulk credential cleanup at the first vault failure", async () => {
    secrets.deleteSecret.mockResolvedValueOnce().mockRejectedValueOnce(new Error("vault failure"));

    await expect(deleteSpaceCredentials(["first", "second", "third"])).rejects.toThrow("vault failure");

    expect(secrets.deleteSecret).toHaveBeenCalledTimes(2);
    expect(secrets.deleteSecret).not.toHaveBeenCalledWith("spaces.credential:third");
  });

  it("does not read the vault before replacing a credential", async () => {
    const persist = vi.fn<() => Promise<void>>().mockRejectedValue(new Error("database failure"));

    await expect(
      commitSpaceCredentialChange("credential-node", { kind: "api-key", secret: "new-secret" }, persist),
    ).rejects.toThrow("database failure");

    expect(secrets.setSecret).toHaveBeenCalledOnce();
    expect(secrets.getSecret).not.toHaveBeenCalled();
    expect(secrets.setSecret).toHaveBeenCalledWith(
      "spaces.credential:credential-node",
      JSON.stringify({ kind: "api-key", secret: "new-secret" }),
    );
  });

  it("persists metadata after the keychain update succeeds", async () => {
    const persist = vi.fn<() => Promise<void>>().mockResolvedValue();

    await commitSpaceCredentialChange(
      "credential-node",
      { kind: "account-password", username: "zane", password: "secret" },
      persist,
    );

    expect(persist).toHaveBeenCalledOnce();
    expect(secrets.deleteSecret).not.toHaveBeenCalled();
  });
});
