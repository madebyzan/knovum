import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent } from "vue";

import type { VoiceProfileHistory } from "../../contracts/generated/VoiceProfileHistory";
import type * as ipc from "../ipc";
import { useVoiceProfileOperations } from "../use-voice-profile-operations";

const mocks = vi.hoisted(() => ({
  listVoiceProfiles: vi.fn<typeof ipc.listVoiceProfiles>(),
  renameVoiceProfile: vi.fn<typeof ipc.renameVoiceProfile>(),
  deleteVoiceProfile: vi.fn<typeof ipc.deleteVoiceProfile>(),
}));
vi.mock("../ipc", () => mocks);
const profile: VoiceProfileHistory = {
  id: "voice",
  name: "Original",
  sourceKind: "reference",
  activeRevisionId: "revision",
  archivedAt: null,
  updatedAt: 1,
  referencedBindingCount: 0,
  revisions: [
    {
      id: "revision",
      revision: 1,
      engineId: "moss_nano",
      modelSpecId: "moss-tts-nano-100m-onnx-bundled-v2",
      language: "en",
      speed: 1,
      sampleRate: 48_000,
      referenceSourceRevision: "sha256:reference",
      designDescription: null,
      createdAt: 1,
    },
  ],
};

function setup() {
  let operations!: ReturnType<typeof useVoiceProfileOperations>;
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Harness = defineComponent({
    setup() {
      operations = useVoiceProfileOperations();
      return () => null;
    },
  });
  const wrapper = mount(Harness, { global: { plugins: [[VueQueryPlugin, { queryClient }]] } });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  return { operations, queryClient };
}

beforeEach(() => {
  vi.resetAllMocks();
  mocks.listVoiceProfiles.mockResolvedValue([profile]);
});

it("loads one global profile collection", async () => {
  const { operations } = setup();
  await flushPromises();
  expect(mocks.listVoiceProfiles).toHaveBeenCalledExactlyOnceWith();
  expect(operations.profiles.value).toEqual([profile]);
});

it("rejects a profile response whose active revision is missing", async () => {
  mocks.listVoiceProfiles.mockResolvedValue([{ ...profile, revisions: [] }]);
  const { operations } = setup();
  await vi.waitFor(() => expect(operations.errors.value).toContain("Voice profile has no active revision"));
  expect(operations.profiles.value).toEqual([]);
});

it("invalidates the global profiles after a successful rename", async () => {
  mocks.renameVoiceProfile.mockResolvedValue();
  mocks.listVoiceProfiles
    .mockResolvedValueOnce([profile])
    .mockResolvedValue([{ ...profile, name: "Renamed", updatedAt: 2 }]);
  const { operations } = setup();
  await flushPromises();
  operations.rename(profile, "Renamed");
  await vi.waitFor(() => expect(operations.profiles.value[0]?.name).toBe("Renamed"));
  expect(mocks.renameVoiceProfile).toHaveBeenCalledWith({
    profileId: profile.id,
    expectedUpdatedAt: profile.updatedAt,
    name: "Renamed",
  });
  expect(operations.busyProfileId.value).toBeUndefined();
});

it("preserves a mutation failure and releases the profile busy state", async () => {
  mocks.renameVoiceProfile.mockRejectedValue(new Error("Rename denied"));
  const { operations } = setup();
  await flushPromises();
  operations.rename(profile, "Renamed");
  await vi.waitFor(() => expect(operations.errors.value).toContain("Rename denied"));
  expect(operations.busyProfileId.value).toBeUndefined();
});

it("refreshes committed deletion while preserving a subsequent cleanup error", async () => {
  const archived = { ...profile, archivedAt: 2, updatedAt: 2 };
  mocks.listVoiceProfiles.mockResolvedValueOnce([archived]).mockResolvedValue([]);
  mocks.deleteVoiceProfile.mockRejectedValue(
    new Error("Voice profile was deleted, but asset cleanup failed"),
  );
  const { operations } = setup();
  await flushPromises();
  operations.remove(archived);
  await vi.waitFor(() => {
    expect(operations.errors.value).toContain("Voice profile was deleted, but asset cleanup failed");
    expect(operations.profiles.value).toEqual([]);
  });
});
