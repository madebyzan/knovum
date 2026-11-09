import { mount, type VueWrapper } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import type { ExternalAgentDescriptor } from "../../../../../shared/contracts/generated/ExternalAgentDescriptor";
import type { ExternalAgentInstallation } from "../../../../../shared/contracts/generated/ExternalAgentInstallation";
import ExternalAgentAvailability from "../ExternalAgentAvailability.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const agent: ExternalAgentDescriptor = {
  kind: "claude_code",
  displayName: "Claude Code",
  executableNames: ["claude"],
  help: {
    connection: "executable_cli",
    documentationUrl: "https://example.com",
    setupUrl: "https://example.com/setup",
  },
};

function mountAgent(
  overrides: Partial<{
    enabled: boolean;
    checking: boolean;
    checkFailed: boolean;
    installation: ExternalAgentInstallation | undefined;
  }>,
) {
  return mount(ExternalAgentAvailability, {
    props: {
      agent,
      enabled: false,
      checking: false,
      checkFailed: false,
      installation: { kind: "claude_code", detected: true, executablePath: "/bin/claude" },
      ...overrides,
    },
  });
}

function toggleButton(wrapper: VueWrapper) {
  return wrapper.findAll("button").find((button) => /settings\.agents\.(enable|disable)/.test(button.text()));
}

describe("ExternalAgentAvailability", () => {
  it("renders agent display name and enables toggle", async () => {
    const wrapper = mountAgent({});
    expect(wrapper.text()).toContain("Claude Code");
    const button = toggleButton(wrapper)!;
    expect(button.text()).toContain("settings.agents.enable");
    await button.trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([[true]]);
    wrapper.unmount();
  });

  it("emits disable when already enabled", async () => {
    const wrapper = mountAgent({ enabled: true });
    const button = toggleButton(wrapper)!;
    expect(button.text()).toContain("settings.agents.disable");
    await button.trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([[false]]);
    wrapper.unmount();
  });

  it("blocks enabling an agent without a command entry and shows connection help", () => {
    const wrapper = mountAgent({
      installation: { kind: "claude_code", detected: false, executablePath: null },
    });
    expect(toggleButton(wrapper)!.attributes("disabled")).toBeDefined();
    expect(wrapper.text()).toContain("settings.agents.entry.missing");
    wrapper.unmount();
  });

  it("blocks enabling when only the application was found", () => {
    const wrapper = mountAgent({
      installation: { kind: "claude_code", detected: true, executablePath: null },
    });
    expect(toggleButton(wrapper)!.attributes("disabled")).toBeDefined();
    expect(wrapper.text()).toContain("settings.agents.entry.applicationOnly");
    wrapper.unmount();
  });

  it("keeps disable available for an enabled agent that lost its command entry", async () => {
    const wrapper = mountAgent({
      enabled: true,
      installation: { kind: "claude_code", detected: false, executablePath: null },
    });
    const button = toggleButton(wrapper)!;
    expect(button.attributes("disabled")).toBeUndefined();
    await button.trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([[false]]);
    wrapper.unmount();
  });

  it("blocks enabling while the installation check is running or failed", () => {
    const checking = mountAgent({ checking: true, installation: undefined });
    expect(toggleButton(checking)!.attributes("disabled")).toBeDefined();
    checking.unmount();

    const failed = mountAgent({ checkFailed: true, installation: undefined });
    expect(toggleButton(failed)!.attributes("disabled")).toBeDefined();
    expect(failed.text()).toContain("settings.agents.status.checkFailed");
    failed.unmount();
  });
});
