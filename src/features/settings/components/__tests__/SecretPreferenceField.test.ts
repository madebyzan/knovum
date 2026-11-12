import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const isSecretConfigured = vi.hoisted(() => vi.fn<(key: string) => Promise<boolean>>());
const setSecret = vi.hoisted(() => vi.fn<(key: string, value: string) => Promise<void>>());
const deleteSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<void>>());

vi.mock("../../../../foundation/secrets/secret-store", () => ({
  isSecretConfigured,
  setSecret,
  deleteSecret,
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({
  toast: { error: vi.fn<(message: string) => void>(), success: vi.fn<(message: string) => void>() },
}));

import SecretPreferenceField from "../SecretPreferenceField.vue";

describe("SecretPreferenceField", () => {
  beforeEach(() => {
    isSecretConfigured.mockReset().mockResolvedValue(false);
    setSecret.mockReset().mockResolvedValue(undefined);
    deleteSecret.mockReset().mockResolvedValue(undefined);
  });

  it("reads configuration without changing the credential when mounted", async () => {
    mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    expect(isSecretConfigured).toHaveBeenCalledWith("ai.key");
    expect(setSecret).not.toHaveBeenCalled();
    expect(deleteSecret).not.toHaveBeenCalled();
  });

  it("writes a trimmed replacement when input is blurred", async () => {
    const wrapper = mount(SecretPreferenceField, {
      props: { label: "API key", secretKey: "ai.key" },
    });
    const input = wrapper.get("input");
    await input.setValue("  replacement  ");
    await input.trigger("blur");
    expect(setSecret).toHaveBeenCalledWith("ai.key", "replacement");
  });

  it("deletes the stored credential when Remove button is clicked", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = mount(SecretPreferenceField, {
      props: { label: "API key", secretKey: "ai.key" },
    });
    await flushPromises();
    const removeBtn = wrapper
      .findAll("button")
      .find((button) => button.attributes("aria-label") === "settings.credentials.remove");
    expect(removeBtn).toBeDefined();
    await removeBtn!.trigger("click");
    expect(deleteSecret).toHaveBeenCalledWith("ai.key");
  });
});
