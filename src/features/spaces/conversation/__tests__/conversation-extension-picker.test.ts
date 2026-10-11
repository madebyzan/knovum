import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { createMemoryHistory, createRouter } from "vue-router";

import { i18n } from "../../../../foundation/i18n/instance";
import type { AgentExtensionAvailability } from "../../../../shared/contracts/generated/AgentExtensionAvailability";
import AgentExtensionPicker from "../presentation/AgentExtensionPicker.vue";

describe("unavailable conversation extensions", () => {
  it("retains the draft until the user explicitly clears revoked extension choices", async () => {
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: "/", component: {} }] });
    const availability: AgentExtensionAvailability = {
      plugins: [],
      skills: [],
      connections: [],
      connectors: [],
      diagnostics: [],
      defaultSelection: { pluginIds: [], skillIds: [], connections: [] },
    };
    const wrapper = mount(AgentExtensionPicker, {
      props: {
        modelValue: {
          pluginIds: [],
          skillIds: ["revoked-skill"],
          connections: [
            {
              connectionId: "revoked-connection",
              toolNames: [],
              resourceUris: [],
              resourceTemplates: [],
              promptNames: [],
            },
          ],
        },
        availability,
        disabled: false,
      },
      global: {
        plugins: [i18n, router],
        stubs: {
          UiPopover: { template: "<div><slot name='trigger' /><slot /></div>" },
          UiScrollArea: { template: "<div><slot /></div>" },
        },
      },
    });
    try {
      expect(wrapper.find('[role="alert"]').exists()).toBe(true);
      expect(wrapper.emitted("update:modelValue")).toBeUndefined();
      await wrapper.get('[data-testid="clear-conversation-extensions"]').trigger("click");
      expect(wrapper.emitted("update:modelValue")).toEqual([
        [{ pluginIds: [], skillIds: [], connections: [] }],
      ]);
    } finally {
      wrapper.unmount();
    }
  });
});

it("selects a complete plugin once without exposing its Skills or services as individual choices", async () => {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: "/", component: {} }] });
  const plugin = {
    id: "package",
    enabled: true,
    builtin: true,
    publication: null,
    source: { type: "directory" as const, path: "/package" },
    revision: {
      digest: "one",
      sourceRevision: "one",
      components: [],
      connectors: [],
      fileCount: 4,
      byteLength: 4,
      manifest: {
        $schema: "https://test/plugin.schema.json",
        name: "Knowledge",
        description: "Complete knowledge workflow",
        version: null,
        author: null,
        homepage: null,
        repository: null,
        license: null,
        keywords: null,
        extensions: null,
      },
    },
  };
  const wrapper = mount(AgentExtensionPicker, {
    props: {
      modelValue: { pluginIds: [], skillIds: [], connections: [] },
      disabled: false,
      availability: {
        plugins: [plugin],
        skills: [],
        connectors: [],
        connections: [],
        diagnostics: [],
        defaultSelection: { pluginIds: [plugin.id], skillIds: [], connections: [] },
      },
    },
    global: {
      plugins: [i18n, router],
      stubs: {
        UiPopover: { template: "<div><slot name='trigger' /><slot /></div>" },
        UiScrollArea: { template: "<div><slot /></div>" },
      },
    },
  });
  try {
    expect(wrapper.findAll('[role="checkbox"]')).toHaveLength(1);
    await wrapper.get('[data-testid="conversation-plugins-package"]').trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([
      [{ pluginIds: ["package"], skillIds: [], connections: [] }],
    ]);
    await wrapper.setProps({ modelValue: { pluginIds: ["package"], skillIds: [], connections: [] } });
    expect(wrapper.get('[data-testid="conversation-extension-picker"]').text()).toBe("1");
  } finally {
    wrapper.unmount();
  }
});
