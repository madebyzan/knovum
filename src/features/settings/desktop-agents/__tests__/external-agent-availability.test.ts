import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import type { ExternalAgentDescriptor } from "../../../../shared/contracts/generated/ExternalAgentDescriptor";
import ExternalAgentAvailability from "../components/ExternalAgentAvailability.vue";

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

describe("ExternalAgentAvailability", () => {
  it("renders agent display name and enables toggle", async () => {
    const wrapper = mount(ExternalAgentAvailability, {
      props: {
        agent,
        enabled: false,
        checking: false,
        checkFailed: false,
        installation: { kind: "claude_code", detected: true, executablePath: "/bin/claude" },
      },
    });
    expect(wrapper.text()).toContain("Claude Code");
    const button = wrapper.find("button");
    expect(button.text()).toContain("settings.agents.enable");
    await button.trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([[true]]);
    wrapper.unmount();
  });

  it("emits disable when already enabled", async () => {
    const wrapper = mount(ExternalAgentAvailability, {
      props: {
        agent,
        enabled: true,
        checking: false,
        checkFailed: false,
        installation: { kind: "claude_code", detected: true, executablePath: "/bin/claude" },
      },
    });
    const button = wrapper.find("button");
    expect(button.text()).toContain("settings.agents.disable");
    await button.trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([[false]]);
    wrapper.unmount();
  });
});
