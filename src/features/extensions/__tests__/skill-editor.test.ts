import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../shared/contracts/settings-leave";
import { activeConfirmation, resolveConfirmation } from "../../../shared/ui/confirm-action";
import SkillOverview from "../skills/SkillOverview.vue";
import SkillSettings from "../skills/SkillSettings.vue";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, input?: Record<string, unknown>) => Promise<unknown>>(),
}));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
const skill = {
  id: "personal",
  pluginId: null,
  name: "weekly-review",
  description: "Review notes",
  digest: "one",
  enabled: true,
  requirements: [],
  includedTools: [],
  unsupportedRequirements: [],
};
let wrapper: ReturnType<typeof mount>;
let queryClient: QueryClient;
let leaveParticipant: SettingsLeaveParticipant;
beforeEach(async () => {
  mocks.invoke.mockReset();
  mocks.invoke.mockImplementation(async (command) => {
    if (command === "plugin:extensions|list_plugin_installations") return [];
    if (command === "plugin:extensions|get_agent_extension_availability")
      return {
        skills: [skill],
        connections: [],
        connectors: [],
        diagnostics: [],
        defaultSelection: { skillIds: [], connections: [] },
      };
    if (command === "plugin:extensions|list_skills") return [skill];
    if (command === "plugin:extensions|read_skill")
      return { skill, markdown: "Original instructions", files: [] };
    throw new Error(`Unexpected command: ${command}`);
  });
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  wrapper = mount(SkillSettings, {
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: {
          register: (participant: SettingsLeaveParticipant) => {
            leaveParticipant = participant;
            return () => {};
          },
          requestLeave: async () => {},
        },
      },
      mocks: { $t: (key: string) => key },
      stubs: { UiDialog: { props: ["open"], template: '<div v-if="open"><slot /></div>' } },
    },
  });
  await flushPromises();
});
afterEach(() => {
  resolveConfirmation(false);
  wrapper.unmount();
  queryClient.clear();
});

it("keeps the editor closed until requested and distinguishes empty search results", async () => {
  expect(wrapper.find('[data-testid="skill-content-editor"]').exists()).toBe(false);
  await wrapper.setProps({ search: "not-found" });
  expect(wrapper.text()).toContain("extensions.plugins.search.empty");
  expect(wrapper.find('[data-testid="skill-personal"]').exists()).toBe(false);
  await wrapper.setProps({ search: "" });
  expect(wrapper.find('[data-testid="skill-personal"]').exists()).toBe(true);
});

it("disables and re-enables the selected skill through explicit IPC intents", async () => {
  let isEnabled = true;
  mocks.invoke.mockImplementation(async (command) => {
    if (command === "plugin:extensions|list_plugin_installations") return [];
    if (command === "plugin:extensions|get_agent_extension_availability")
      return {
        skills: isEnabled ? [skill] : [],
        connections: [],
        connectors: [],
        diagnostics: [],
        defaultSelection: { skillIds: [], connections: [] },
      };
    if (command === "plugin:extensions|list_skills") return [{ ...skill, enabled: isEnabled }];
    if (command === "plugin:extensions|disable_skill") isEnabled = false;
    else if (command === "plugin:extensions|enable_skill") isEnabled = true;
    else throw new Error(`Unexpected command: ${command}`);
  });

  await wrapper.get('[data-testid="toggle-skill-personal"]').trigger("click");
  await flushPromises();
  expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|disable_skill", { skillId: "personal" });
  expect(wrapper.get('[data-testid="toggle-skill-personal"]').attributes("aria-checked")).toBe("false");

  await wrapper.get('[data-testid="toggle-skill-personal"]').trigger("click");
  await flushPromises();
  expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|enable_skill", { skillId: "personal" });
  expect(wrapper.get('[data-testid="toggle-skill-personal"]').attributes("aria-checked")).toBe("true");
});

it("preserves edits when discard is cancelled and closes only after confirming discard", async () => {
  wrapper.getComponent(SkillOverview).vm.$emit("edit");
  await flushPromises();
  const editor = wrapper.get('[data-testid="skill-content-editor"]');
  await editor.get("textarea").setValue("Unsaved instructions");
  await editor.get('button[type="button"]').trigger("click");
  expect(activeConfirmation.value).toBeDefined();
  resolveConfirmation(false);
  await flushPromises();
  expect(wrapper.get("textarea").element).toHaveProperty("value", "Unsaved instructions");
  await editor.get('button[type="button"]').trigger("click");
  resolveConfirmation(true);
  await flushPromises();
  expect(wrapper.find('[data-testid="skill-content-editor"]').exists()).toBe(false);
  expect(
    mocks.invoke.mock.calls.some(([command]) => command === "plugin:extensions|save_skill_revision"),
  ).toBe(false);
});

it("does not prompt to discard edits after a successful save", async () => {
  const original = mocks.invoke.getMockImplementation()!;
  mocks.invoke.mockImplementation(async (command, input) => {
    if (command === "plugin:extensions|save_skill_revision")
      return { value: { ...skill, digest: "saved" }, cleanupErrors: [] };
    return original(command, input);
  });
  wrapper.getComponent(SkillOverview).vm.$emit("edit");
  await flushPromises();
  await wrapper.get("textarea").setValue("Saved instructions");
  await wrapper.get('[data-testid="skill-content-editor"]').trigger("submit");
  await flushPromises();
  expect(wrapper.find('[data-testid="skill-content-editor"]').exists()).toBe(false);
  await expect(leaveParticipant.prepare()).resolves.toBe(true);
  expect(activeConfirmation.value).toBeUndefined();
});
