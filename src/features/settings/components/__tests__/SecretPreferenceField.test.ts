import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const persistOptionalSecret = vi.hoisted(() => vi.fn<(key: string, value: string) => Promise<void>>());
const getSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<string | null>>());

vi.mock("../../../../foundation/secrets/secret-store", () => ({ getSecret, persistOptionalSecret }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({
  toast: { error: vi.fn<(message: string) => void>(), success: vi.fn<(message: string) => void>() },
}));

import SecretPreferenceField from "../SecretPreferenceField.vue";

describe("SecretPreferenceField", () => {
  beforeEach(() => {
    persistOptionalSecret.mockReset().mockResolvedValue(undefined);
    getSecret.mockReset().mockResolvedValue(null);
  });

  it("does not read or write the credential when mounted", () => {
    mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });

    expect(persistOptionalSecret).not.toHaveBeenCalled();
  });

  it("writes a replacement when editing finishes", async () => {
    const wrapper = mount(SecretPreferenceField, {
      props: { label: "API key", secretKey: "ai.key" },
    });
    const input = wrapper.get("input");
    await input.setValue("  replacement  ");
    expect(persistOptionalSecret).not.toHaveBeenCalled();
    await input.trigger("blur");

    expect(persistOptionalSecret).toHaveBeenCalledWith("ai.key", "  replacement  ");
  });

  it("deletes the credential when a cleared field loses focus", async () => {
    const wrapper = mount(SecretPreferenceField, {
      props: { label: "API key", secretKey: "ai.key" },
    });
    const input = wrapper.get("input");
    await input.setValue("replacement");
    await input.setValue("");
    await input.trigger("blur");

    expect(persistOptionalSecret).toHaveBeenCalledWith("ai.key", "");
  });
});
