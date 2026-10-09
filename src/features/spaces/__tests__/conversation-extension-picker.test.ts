import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { createMemoryHistory, createRouter } from "vue-router";

import { i18n } from "../../../foundation/i18n/i18n";
import type { AgentExtensionAvailability } from "../../../shared/contracts/generated/AgentExtensionAvailability";
import AgentExtensionPicker from "../presentation/conversation/AgentExtensionPicker.vue";

describe("unavailable conversation extensions", () => {
  it("retains the draft until the user explicitly clears revoked extension choices", async () => {
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: "/", component: {} }] });
    const availability: AgentExtensionAvailability = {
      skills: [],
      connections: [],
      connectors: [],
      diagnostics: [],
      defaultSelection: { skillIds: [], connections: [] },
    };
    const wrapper = mount(AgentExtensionPicker, {
      props: {
        modelValue: {
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
      expect(wrapper.emitted("update:modelValue")).toEqual([[{ skillIds: [], connections: [] }]]);
    } finally {
      wrapper.unmount();
    }
  });
});
