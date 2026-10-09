import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ComposerTranslation } from "vue-i18n";

import type { MaintenanceReceipt } from "../../contracts/generated/MaintenanceReceipt";
import { showBackupMaintenanceNotice } from "../maintenance-notice";

const calls = vi.hoisted(() => ({
  invoke: vi.fn<(command: string) => Promise<unknown>>(),
  success: vi.fn<(...args: unknown[]) => void>(),
  warning: vi.fn<(...args: unknown[]) => void>(),
  error: vi.fn<(...args: unknown[]) => void>(),
  log: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
}));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: calls.invoke }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: calls.log }));
vi.mock("vue-sonner", () => ({ toast: calls }));
const t = ((key: string) => key) as ComposerTranslation;

describe("backup maintenance notice", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows credential reauthorization together with a committed restore cleanup warning", async () => {
    const receipt: MaintenanceReceipt = {
      operation: "restore",
      status: "success",
      message: "Restored",
      cleanupWarning: "Cleanup needs attention",
      credentialsReconfigurationRequired: true,
    };
    calls.invoke.mockResolvedValue(receipt);
    await showBackupMaintenanceNotice(t);
    expect(calls.warning).toHaveBeenCalledWith(
      "settings.data.receipt.restore.success",
      expect.objectContaining({
        description:
          "settings.data.receipt.exclusions\nsettings.data.receipt.credentials\nCleanup needs attention",
        duration: Infinity,
      }),
    );
    expect(calls.success).not.toHaveBeenCalled();
  });

  it("keeps an invalid current receipt visible as a read failure", async () => {
    calls.invoke.mockResolvedValue({
      operation: "legacy_restore",
      status: "success",
      message: "invalid",
      cleanupWarning: null,
      credentialsReconfigurationRequired: false,
    });
    await showBackupMaintenanceNotice(t);
    expect(calls.error).toHaveBeenCalledWith(
      "settings.data.receipt.readFailed",
      expect.objectContaining({ duration: Infinity }),
    );
    expect(calls.warning).not.toHaveBeenCalled();
    expect(calls.success).not.toHaveBeenCalled();
    expect(calls.invoke).not.toHaveBeenCalledWith("plugin:backup|acknowledge_maintenance_receipt");
  });

  it("shows the initiating operation failure and leaves its receipt for explicit dismissal", async () => {
    calls.invoke.mockResolvedValue({
      operation: "backup",
      status: "failed",
      message: "Selected destination denied",
      cleanupWarning: null,
      credentialsReconfigurationRequired: false,
    } satisfies MaintenanceReceipt);
    await showBackupMaintenanceNotice(t);
    expect(calls.error).toHaveBeenCalledWith(
      "settings.data.receipt.backup.failed",
      expect.objectContaining({
        description: "Selected destination denied",
        duration: Infinity,
      }),
    );
    expect(calls.invoke).toHaveBeenCalledOnce();
  });
});
