import { describe, expect, it } from "vitest";

import type { UpdateMetadata } from "../../../shared/contracts/generated/UpdateMetadata";
import { releaseNotesBody, updateTransfer } from "../update-presentation";

const release: UpdateMetadata = { version: "0.0.3", currentVersion: "0.0.2", notes: "" };
const MiB = 1024 * 1024;

describe("updateTransfer", () => {
  it("reads a sized download as received of total with the locale's percentage", () => {
    expect(
      updateTransfer({ status: "downloading", release, downloaded: 12 * MiB, total: 48 * MiB }, "en"),
    ).toEqual({
      value: 12 * MiB,
      max: 48 * MiB,
      amount: "12 MiB / 48 MiB",
      percent: "25%",
    });
    expect(
      updateTransfer({ status: "downloading", release, downloaded: 12 * MiB, total: 48 * MiB }, "fr")
        ?.percent,
    ).toBe("25\u00a0%");
  });

  it("never rounds an unfinished download up to complete", () => {
    expect(
      updateTransfer({ status: "downloading", release, downloaded: 999, total: 1000 }, "en")?.percent,
    ).toBe("99%");
  });

  it("sweeps while the size is unknown and while the signature is checked", () => {
    expect(
      updateTransfer({ status: "downloading", release, downloaded: 3 * MiB, total: null }, "en"),
    ).toEqual({
      value: null,
      max: 1,
      amount: "3.0 MiB",
      percent: undefined,
    });
    expect(updateTransfer({ status: "verifying", release }, "en")?.value).toBeNull();
  });

  it("shows no transfer before a download or after it settles", () => {
    expect(updateTransfer({ status: "available", release }, "en")).toBeUndefined();
    expect(updateTransfer({ status: "ready", release }, "en")).toBeUndefined();
  });
});

describe("releaseNotesBody", () => {
  it("leaves out the release's own title, which the panel already names", () => {
    expect(releaseNotesBody("# Knovum 0.0.3\n\nCreating is no longer a dialog.\n\n## Fixes\n\n- One")).toBe(
      "Creating is no longer a dialog.\n\n## Fixes\n\n- One",
    );
  });

  it("keeps notes that do not open with a title", () => {
    expect(releaseNotesBody("## Fixes\n\n- One\n")).toBe("## Fixes\n\n- One");
  });
});
