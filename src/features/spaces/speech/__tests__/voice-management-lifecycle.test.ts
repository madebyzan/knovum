import { flushPromises } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import type * as ipc from "../ipc";
import { useVoiceManagement } from "../use-voice-management";

const mocks = vi.hoisted(() => ({
  listVoiceProfiles: vi.fn<typeof ipc.listVoiceProfiles>(),
  renameVoiceProfile: vi.fn<typeof ipc.renameVoiceProfile>(),
}));
vi.mock("../ipc", () => mocks);
const profile: VoiceProfileHistory = {
  id: "voice",
  spaceId: "first",
  name: "Original",
  sourceKind: "reference",
  activeRevisionId: "revision",
  archivedAt: null,
  updatedAt: 1,
  referencedSegmentCount: 0,
  revisions: [],
};
const scopes: ReturnType<typeof effectScope>[] = [];
async function setup() {
  const owner = shallowRef("first");
  const changed = vi.fn<() => void>();
  const scope = effectScope();
  scopes.push(scope);
  const management = scope.run(() => useVoiceManagement(() => owner.value, changed))!;
  await flushPromises();
  return { management, owner, changed, scope };
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.listVoiceProfiles.mockResolvedValue([profile]);
});
afterEach(() => scopes.splice(0).forEach((scope) => scope.stop()));

it("clears the previous Space's voices while the next Space is loading", async () => {
  const { management, owner } = await setup();
  mocks.listVoiceProfiles.mockReturnValue(new Promise(() => {}));
  owner.value = "second";
  await flushPromises();
  expect(management.profiles.value).toEqual([]);
});

it.each(["space-change", "dispose"] as const)("ignores a late mutation failure after %s", async (action) => {
  const { management, owner, changed, scope } = await setup();
  let failRename!: (cause: unknown) => void;
  mocks.renameVoiceProfile.mockReturnValue(
    new Promise((_resolve, reject) => {
      failRename = reject;
    }),
  );
  const pending = management.rename(profile, "Renamed");
  if (action === "dispose") scope.stop();
  else {
    mocks.listVoiceProfiles.mockResolvedValue([]);
    owner.value = "second";
  }
  await flushPromises();
  failRename(new Error("Old rename failed"));
  await pending;
  expect(management.error.value).toBeUndefined();
  expect(changed).not.toHaveBeenCalled();
});

it("does not reload a new Space after an old mutation succeeds", async () => {
  const { management, owner, changed } = await setup();
  let finishRename!: () => void;
  mocks.renameVoiceProfile.mockReturnValue(
    new Promise((resolve) => {
      finishRename = () => resolve();
    }),
  );
  const pending = management.rename(profile, "Renamed");
  mocks.listVoiceProfiles.mockResolvedValue([]);
  owner.value = "second";
  await flushPromises();
  finishRename();
  await pending;
  expect(mocks.listVoiceProfiles).toHaveBeenCalledTimes(2);
  expect(management.profiles.value).toEqual([]);
  expect(changed).not.toHaveBeenCalled();
});

it("reloads and emits a change when the current voice is renamed", async () => {
  const { management, changed } = await setup();
  mocks.renameVoiceProfile.mockResolvedValue(undefined);
  mocks.listVoiceProfiles.mockResolvedValue([{ ...profile, name: "Renamed" }]);
  await management.rename(profile, "Renamed");
  expect(management.profiles.value[0]?.name).toBe("Renamed");
  expect(management.busyProfileId.value).toBeUndefined();
  expect(changed).toHaveBeenCalledOnce();
});

it("preserves a current mutation failure and releases its busy state", async () => {
  const { management, changed } = await setup();
  mocks.renameVoiceProfile.mockRejectedValue(new Error("Rename denied"));
  await management.rename(profile, "Renamed");
  expect(management.error.value).toBe("Rename denied");
  expect(management.busyProfileId.value).toBeUndefined();
  expect(changed).not.toHaveBeenCalled();
});
