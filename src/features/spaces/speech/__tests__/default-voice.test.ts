import { beforeEach, describe, expect, it, vi } from "vitest";

import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import {
  loadDefaultVoiceProfileId,
  saveDefaultVoiceProfileId,
  useDefaultVoiceProfile,
} from "../default-voice";
import { defaultVoiceRevisionId } from "../voice-options";

const mockStore = vi.hoisted(() => ({
  get: vi.fn<(key: string) => Promise<unknown>>(),
  set: vi.fn<(key: string, val: unknown) => Promise<void>>(),
  save: vi.fn<() => Promise<void>>(),
  stored: undefined as unknown,
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  LazyStore: class {
    get = mockStore.get;
    set = mockStore.set;
    save = mockStore.save;
  },
}));

const mockVoices: VoiceLibraryRevision[] = [
  {
    profileId: "profile-1",
    name: "Voice 1",
    sourceKind: "reference",
    revisionId: "rev-1",
    revision: 1,
    isActive: true,
    supportsDeliveryInstruction: false,
    engineId: "moss_nano",
    modelSpecId: "moss",
    language: "zh-CN",
    updatedAt: 1,
  },
  {
    profileId: "profile-2",
    name: "Voice 2",
    sourceKind: "reference",
    revisionId: "rev-2",
    revision: 1,
    isActive: true,
    supportsDeliveryInstruction: true,
    engineId: "voxcpm2",
    modelSpecId: "voxcpm2",
    language: "zh-CN",
    updatedAt: 2,
  },
];

describe("default voice profile persistence", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStore.stored = undefined;
    mockStore.get.mockImplementation(async () => mockStore.stored);
    mockStore.set.mockImplementation(async (_key, val) => {
      mockStore.stored = val;
    });
    mockStore.save.mockResolvedValue(undefined);
  });

  it("loads null when no default voice is set", async () => {
    expect(await loadDefaultVoiceProfileId()).toBeNull();
  });

  it("persists and reads configured default voice profile id", async () => {
    await saveDefaultVoiceProfileId("profile-2");
    expect(mockStore.set).toHaveBeenCalledWith("speech.defaultVoiceProfileId", {
      schemaVersion: 1,
      value: "profile-2",
    });
    expect(mockStore.save).toHaveBeenCalled();
    expect(await loadDefaultVoiceProfileId()).toBe("profile-2");
  });

  it("resolves revision id matching default profile id or falls back", () => {
    expect(defaultVoiceRevisionId(mockVoices, "profile-2")).toBe("rev-2");
    expect(defaultVoiceRevisionId(mockVoices, "profile-1")).toBe("rev-1");
    expect(defaultVoiceRevisionId(mockVoices, "non-existent")).toBe("rev-1");
    expect(defaultVoiceRevisionId(mockVoices, null)).toBe("rev-1");
  });

  it("provides reactive default voice profile state", async () => {
    const { defaultVoiceProfileId, setDefaultVoiceProfile } = useDefaultVoiceProfile();
    await setDefaultVoiceProfile("profile-2");
    expect(defaultVoiceProfileId.value).toBe("profile-2");
  });
});
