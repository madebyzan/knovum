import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

import { ApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import {
  type SettingsLeaveCoordinator,
  useSettingsLeaveContext,
} from "../../../../shared/contracts/settings-leave";
import { useSettingsLeaveCoordinator } from "../use-settings-leave-coordinator";

vi.mock("../../../../foundation/lifecycle/window-close-coordinator", () => ({
  registerWindowCloseCommit: () => () => {},
}));

it("keeps every draft intact when a later settings participant cancels global leave", async () => {
  let coordinator!: SettingsLeaveCoordinator;
  let firstDraftOpen = true;
  let secondDraftOpen = true;
  const Drafts = defineComponent({
    setup() {
      coordinator = useSettingsLeaveContext();
      coordinator.register({
        prepare: async () => true,
        finalize() {
          firstDraftOpen = false;
        },
      });
      coordinator.register({
        prepare: async () => false,
        finalize() {
          secondDraftOpen = false;
        },
      });
      return () => null;
    },
  });
  const Provider = defineComponent({
    setup() {
      useSettingsLeaveCoordinator();
      return () => h(Drafts);
    },
  });
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: "/settings", component: Provider }],
  });
  await router.push("/settings");
  const wrapper = mount(Provider, { global: { plugins: [router] } });
  onTestFinished(() => wrapper.unmount());

  await expect(coordinator.requestLeave()).rejects.toBeInstanceOf(ApplicationLeaveCancelled);
  expect(firstDraftOpen).toBe(true);
  expect(secondDraftOpen).toBe(true);
});
