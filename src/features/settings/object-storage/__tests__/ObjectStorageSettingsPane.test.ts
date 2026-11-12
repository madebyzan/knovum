import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ObjectStorageSettings } from "../../../../shared/contracts/generated/ObjectStorageSettings";

const saveObjectStorageSettings = vi.hoisted(() =>
  vi.fn<(settings: ObjectStorageSettings) => Promise<void>>(),
);
const loadObjectStorageSettings = vi.hoisted(() => vi.fn<() => Promise<ObjectStorageSettings | null>>());
const removeObjectStorageConfiguration = vi.hoisted(() => vi.fn<() => Promise<void>>());
const testObjectStorageConnection = vi.hoisted(() => vi.fn<() => Promise<void>>());
const isSecretConfigured = vi.hoisted(() => vi.fn<(key: string) => Promise<boolean>>());

vi.mock("../../../../foundation/object-storage/object-storage-settings", () => ({
  OBJECT_STORAGE_ACCESS_KEY_ID: "storage.ak",
  OBJECT_STORAGE_SECRET_ACCESS_KEY: "storage.sk",
  loadObjectStorageSettings,
  saveObjectStorageSettings,
  removeObjectStorageConfiguration,
  testObjectStorageConnection,
}));

vi.mock("../../../../foundation/secrets/secret-store", () => ({
  isSecretConfigured,
  setSecret: vi.fn<(key: string, secret: string) => Promise<void>>(),
  deleteSecret: vi.fn<(key: string) => Promise<void>>(),
}));

vi.mock("vue-i18n", () => ({
  useI18n: () => ({ t: (k: string) => k }),
}));

vi.mock("vue-sonner", () => ({
  toast: { error: vi.fn<(msg: string) => void>(), success: vi.fn<(msg: string) => void>() },
}));

import ObjectStorageSettingsPane from "../ObjectStorageSettingsPane.vue";

describe("ObjectStorageSettingsPane", () => {
  let headerActions: HTMLDivElement;

  beforeEach(() => {
    vi.clearAllMocks();
    headerActions = document.createElement("div");
    headerActions.id = "settings-header-actions";
    document.body.appendChild(headerActions);

    loadObjectStorageSettings.mockResolvedValue(null);
    saveObjectStorageSettings.mockResolvedValue(undefined);
    removeObjectStorageConfiguration.mockResolvedValue(undefined);
    testObjectStorageConnection.mockResolvedValue(undefined);
    isSecretConfigured.mockResolvedValue(false);
  });

  afterEach(() => {
    headerActions.remove();
  });

  it("does not render manual save buttons in the settings pane", async () => {
    const wrapper = mount(ObjectStorageSettingsPane);
    await flushPromises();

    const buttons = wrapper.findAll("button");
    const saveButtons = buttons.filter(
      (btn) =>
        btn.text().includes("common.save") || btn.text().includes("settings.objectStorage.actions.save"),
    );
    expect(saveButtons).toHaveLength(0);
  });

  it("auto-saves configuration when all required fields are filled and blurred", async () => {
    const wrapper = mount(ObjectStorageSettingsPane);
    await flushPromises();

    const inputs = wrapper.findAll("input");
    // Endpoint
    await inputs[0]!.setValue("https://cos.ap-guangzhou.myqcloud.com");
    await inputs[0]!.trigger("blur");
    expect(saveObjectStorageSettings).not.toHaveBeenCalled();

    // Region
    await inputs[1]!.setValue("ap-guangzhou");
    await inputs[1]!.trigger("blur");
    expect(saveObjectStorageSettings).not.toHaveBeenCalled();

    // Bucket
    await inputs[2]!.setValue("my-bucket");
    await inputs[2]!.trigger("blur");

    expect(saveObjectStorageSettings).toHaveBeenCalledWith({
      endpoint: "https://cos.ap-guangzhou.myqcloud.com",
      region: "ap-guangzhou",
      bucket: "my-bucket",
      publicBaseUrl: null,
    });
  });
});
