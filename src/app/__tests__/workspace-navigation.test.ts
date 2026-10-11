import { beforeEach, expect, it, vi } from "vitest";

import { ApplicationLeaveCancelled } from "../../shared/contracts/application-leave";
import { router } from "../router";

const mocks = vi.hoisted(() => ({
  commit: vi.fn<() => Promise<void>>(),
  logError: vi.fn<(message: string) => Promise<void>>(),
  toastError: vi.fn<(message: string) => void>(),
}));

vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.logError }));
vi.mock("vue-sonner", () => ({ toast: { error: mocks.toastError } }));
vi.mock("../../features/spaces/session/space-store", () => ({
  useSpaceStore: () => ({ commitWorkspaceNavigation: mocks.commit }),
}));
vi.mock("../../features/spaces/workflows/action-catalog", () => ({ listSpaceActions: async () => [] }));
vi.mock("../../features/spaces/SpaceView.vue", () => ({ default: { render: () => null } }));
vi.mock("../../features/settings/SettingsView.vue", () => ({ default: { render: () => null } }));

beforeEach(async () => {
  vi.clearAllMocks();
  mocks.commit.mockReset().mockResolvedValue(undefined);
  mocks.logError.mockResolvedValue(undefined);
  await router.push("/spaces");
});

it("settles the resident workspace commit before opening settings", async () => {
  let release!: () => void;
  mocks.commit.mockReturnValueOnce(new Promise<void>((resolve) => (release = resolve)));
  const navigation = router.push("/settings?pane=appearance");
  await vi.waitFor(() => expect(mocks.commit).toHaveBeenCalledOnce());
  expect(router.currentRoute.value.name).toBe("spaces");
  release();
  await navigation;
  expect(router.currentRoute.value.name).toBe("settings");
});

it("keeps the workspace active when its owner cancels leaving", async () => {
  mocks.commit.mockRejectedValueOnce(new ApplicationLeaveCancelled());
  await router.push("/settings");
  expect(router.currentRoute.value.name).toBe("spaces");
  expect(mocks.toastError).not.toHaveBeenCalled();
  expect(mocks.logError).not.toHaveBeenCalled();
});

it("keeps the workspace active and reports an actual commit failure", async () => {
  mocks.commit.mockRejectedValueOnce(new Error("Save failed"));
  await router.push("/settings");
  expect(router.currentRoute.value.name).toBe("spaces");
  expect(mocks.toastError).toHaveBeenCalledWith("Save failed");
  expect(mocks.logError).toHaveBeenCalledWith(expect.stringContaining("Save failed"));
});

it("does not commit the workspace for settings pane changes or returning to it", async () => {
  await router.push("/settings?pane=appearance");
  mocks.commit.mockClear();
  await router.replace("/settings?pane=voices");
  await router.push("/spaces");
  expect(mocks.commit).not.toHaveBeenCalled();
});
