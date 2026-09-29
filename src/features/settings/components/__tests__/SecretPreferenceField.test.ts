import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<string | null>>());
const setSecret = vi.hoisted(() => vi.fn<(key: string, value: string) => Promise<void>>());
const deleteSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<void>>());

vi.mock("../../../../foundation/secrets/secret-store", () => ({ getSecret, setSecret, deleteSecret }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({
  toast: { error: vi.fn<(message: string) => void>(), success: vi.fn<(message: string) => void>() },
}));

import SecretPreferenceField from "../SecretPreferenceField.vue";

describe("SecretPreferenceField", () => {
  beforeEach(() => {
    getSecret.mockReset().mockResolvedValue(null);
    setSecret.mockReset().mockResolvedValue(undefined);
    deleteSecret.mockReset().mockResolvedValue(undefined);
  });

  it("reads configuration without changing the credential when mounted", async () => {
    mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    expect(getSecret).toHaveBeenCalledWith("ai.key");
    expect(setSecret).not.toHaveBeenCalled();
    expect(deleteSecret).not.toHaveBeenCalled();
  });

  it("writes a trimmed replacement only after Save is clicked", async () => {
    const wrapper = mount(SecretPreferenceField, {
      props: { label: "API key", secretKey: "ai.key" },
    });
    const input = wrapper.get("input");
    await input.setValue("  replacement  ");
    await input.trigger("blur");
    expect(setSecret).not.toHaveBeenCalled();
    await wrapper
      .findAll("button")
      .find((button) => button.text() === "settings.ai.saveConnection")!
      .trigger("click");
    expect(setSecret).toHaveBeenCalledWith("ai.key", "replacement");
  });

  it("deletes the stored credential only after Remove is clicked", async () => {
    getSecret.mockResolvedValue("existing");
    const wrapper = mount(SecretPreferenceField, {
      props: { label: "API key", secretKey: "ai.key" },
    });
    await flushPromises();
    await wrapper
      .findAll("button")
      .find((button) => button.text() === "settings.ai.removeSavedKey")!
      .trigger("click");
    expect(deleteSecret).toHaveBeenCalledWith("ai.key");
  });
});
