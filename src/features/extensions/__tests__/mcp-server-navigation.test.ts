import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent } from "vue";
import { createMemoryHistory, createRouter, RouterView } from "vue-router";

import { i18n } from "../../../foundation/i18n/i18n";
import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../shared/contracts/settings-leave";
import { activeConfirmation, resolveConfirmation } from "../../../shared/ui/confirm-action";
import AgentExtensionsSettings from "../AgentExtensionsSettings.vue";
const invoke = vi.hoisted(() => vi.fn<() => Promise<unknown>>(async () => []));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));

it("opens an inline MCP editor and preserves dirty input when Back is cancelled, then leaves on confirmation", async () => {
  const participants = new Set<SettingsLeaveParticipant>();
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: "/settings", component: AgentExtensionsSettings }],
  });
  await router.replace("/settings?pane=extensions&extensionTab=mcp");
  const wrapper = mount(
    defineComponent({
      components: { RouterView },
      template:
        '<div><header><h1>Extensions</h1><div id="settings-header-actions" /></header><RouterView /></div>',
    }),
    {
      attachTo: document.body,
      global: {
        plugins: [router, i18n, [VueQueryPlugin, { queryClient: client }]],
        provide: {
          [settingsLeaveCoordinatorKey as symbol]: {
            register(participant: SettingsLeaveParticipant) {
              participants.add(participant);
              return () => participants.delete(participant);
            },
            async requestLeave() {
              for (const participant of participants)
                if (!(await participant.prepare())) throw new ApplicationLeaveCancelled();
              for (const participant of participants) participant.finalize?.();
            },
          },
        },
      },
    },
  );
  onTestFinished(() => {
    resolveConfirmation(false);
    wrapper.unmount();
    client.clear();
  });
  await flushPromises();
  await wrapper.get('[data-testid="add-mcp-server"]').trigger("click");
  await flushPromises();
  expect(router.currentRoute.value.query.extensionAction).toBe("create");
  expect(wrapper.find('[role="dialog"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="extension-search"]').exists()).toBe(false);
  await wrapper.get('[data-testid="connector-name"]').setValue("Research");
  await wrapper.get('[data-testid="extension-back"]').trigger("click");
  await flushPromises();
  expect(activeConfirmation.value).toBeDefined();
  resolveConfirmation(false);
  await flushPromises();
  expect(router.currentRoute.value.query.extensionAction).toBe("create");
  expect(wrapper.get('[data-testid="connector-name"]').element).toHaveProperty("value", "Research");
  await wrapper.get('[data-testid="extension-back"]').trigger("click");
  await flushPromises();
  resolveConfirmation(true);
  await flushPromises();
  expect(router.currentRoute.value.query.extensionAction).toBeUndefined();
  expect(wrapper.find('[data-testid="connector-editor"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="add-mcp-server"]').exists()).toBe(true);
  await wrapper.get('[data-testid="add-mcp-server"]').trigger("click");
  await flushPromises();
  expect(wrapper.get('[data-testid="connector-name"]').element).toHaveProperty("value", "");
  await wrapper.get('[data-testid="extension-back"]').trigger("click");
  await flushPromises();
  expect(router.currentRoute.value.query.extensionAction).toBeUndefined();
  expect(activeConfirmation.value).toBeUndefined();
});
