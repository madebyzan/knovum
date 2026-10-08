import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import { activeConfirmation, resolveConfirmation } from "../../../shared/ui/confirm-action";
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
beforeEach(async () => {
  mocks.invoke.mockReset();
  mocks.invoke.mockImplementation(async (command) => {
    if (command === "plugin:extensions|list_skills") return [skill];
    if (command === "plugin:extensions|read_skill")
      return { skill, markdown: "Original instructions", files: [] };
    throw new Error(`Unexpected command: ${command}`);
  });
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  wrapper = mount(SkillSettings, {
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
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
  await wrapper.get('[data-testid="skill-search"] input').setValue("not-found");
  expect(wrapper.text()).toContain("extensions.plugins.search.empty");
  expect(wrapper.find('[data-testid="skill-personal"]').exists()).toBe(false);
  await wrapper.get('[data-testid="skill-search"] input').setValue("");
  expect(wrapper.find('[data-testid="skill-personal"]').exists()).toBe(true);
});

it("preserves edits when discard is cancelled and closes only after confirming discard", async () => {
  await wrapper.get('[data-testid="edit-skill-personal"]').trigger("click");
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
    mocks.invoke.mock.calls.every(
      ([command]) =>
        command === "plugin:extensions|list_skills" || command === "plugin:extensions|read_skill",
    ),
  ).toBe(true);
});
