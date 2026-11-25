import { describe, expect, it } from "vitest";

import type { UpdateMetadata } from "../../../shared/contracts/generated/UpdateMetadata";
import {
  canCheckForUpdate,
  getUpdateAttentionVersion,
  getUpdateRelease,
  getUpdateStatusMessageKey,
  isUpdateAttentionStatus,
  isUpdateToolbarVisible,
  releaseNotesBody,
  updateTransfer,
} from "../update-presentation";

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

describe("update status helpers", () => {
  it("distinguishes toolbar visibility, sidebar attention, and checkability", () => {
    expect(isUpdateToolbarVisible(null)).toBe(false);
    expect(isUpdateToolbarVisible({ status: "current" })).toBe(false);
    expect(isUpdateToolbarVisible({ status: "available", release })).toBe(true);
    expect(isUpdateToolbarVisible({ status: "downloading", release, downloaded: 1, total: 2 })).toBe(true);
    expect(isUpdateToolbarVisible({ status: "verifying", release })).toBe(true);
    expect(isUpdateToolbarVisible({ status: "ready", release })).toBe(true);

    expect(isUpdateAttentionStatus({ status: "available", release })).toBe(true);
    expect(isUpdateAttentionStatus({ status: "ready", release })).toBe(true);
    expect(isUpdateAttentionStatus({ status: "downloading", release, downloaded: 1, total: 2 })).toBe(false);
    expect(getUpdateAttentionVersion({ status: "available", release })).toBe("0.0.3");
    expect(getUpdateAttentionVersion({ status: "ready", release })).toBe("0.0.3");
    expect(getUpdateAttentionVersion({ status: "downloading", release, downloaded: 1, total: 2 })).toBeNull();

    expect(canCheckForUpdate(null)).toBe(false);
    expect(canCheckForUpdate({ status: "disabled" })).toBe(false);
    expect(canCheckForUpdate({ status: "idle" })).toBe(true);
    expect(canCheckForUpdate({ status: "current" })).toBe(true);
    expect(canCheckForUpdate({ status: "failed", reason: "check" })).toBe(true);

    expect(getUpdateRelease({ status: "available", release })).toEqual(release);
    expect(getUpdateRelease({ status: "current" })).toBeNull();
    expect(getUpdateStatusMessageKey(null)).toBe("updates.connection.loading");
    expect(getUpdateStatusMessageKey({ status: "failed", reason: "signature" })).toBe(
      "updates.failure.signature",
    );
    expect(getUpdateStatusMessageKey({ status: "ready", release })).toBe("updates.status.ready");
  });
});
